use argon2::{
    Argon2, PasswordHasher,
    password_hash::{PasswordHash, PasswordVerifier, SaltString, rand_core::OsRng},
};
use atlas::http_api::{err, ok};
use axum::{
    Json, Router,
    response::{Html, IntoResponse},
    routing::{get, post},
};
use axum_extra::extract::CookieJar;
use cookie::{Cookie, time::Duration};
use redis::AsyncTypedCommands;
use serde::Deserialize;

use crate::{
    db::User,
    error::{Error, R, Result},
    statics::{cfg, redis},
};

const SESSION_EXPIRE: i64 = 60; // 60 days

#[derive(Deserialize)]
pub struct UserFromPage {
    pub username: String,
    pub password: String,
}

pub async fn login_page() -> Html<&'static str> {
    Html(include_str!("../html/login.html"))
}

pub async fn register_page() -> Html<&'static str> {
    Html(include_str!("../html/register.html"))
}

pub async fn register(Json(req): Json<UserFromPage>) -> R<()> {
    let cfg = cfg().await?;

    if !cfg.allow_register {
        return err(403, "User registration is disabled");
    }

    if User::find_by_username(&req.username).await?.is_some() {
        return err(400, "this username is already in use.");
    }

    let salt = SaltString::generate(&mut OsRng);
    let password_hash = Argon2::default()
        .hash_password(req.password.as_bytes(), &salt)?
        .to_string();

    User::create(&req.username, &password_hash).await?;
    ok(())
}

pub async fn login(jar: CookieJar, Json(req): Json<UserFromPage>) -> Result<impl IntoResponse> {
    let user = User::find_by_username(&req.username)
        .await?
        .ok_or_else(|| Error::new(401, "Invalid username or password"))?;

    let parsed_hash = PasswordHash::new(&user.password_hash)?;

    let valid = Argon2::default()
        .verify_password(req.password.as_bytes(), &parsed_hash)
        .is_ok();

    if !valid {
        return Err(Error::new(401, "Invalid username or password"));
    }

    let key = Redis::set_session(&user.id.to_string()).await?;

    ok(()).map(|resp| {
        let cookie = Cookie::build(("token", key))
            .secure(true)
            .http_only(true)
            .max_age(Duration::days(SESSION_EXPIRE))
            .path("/");
        (jar.add(cookie), resp)
    })
}

pub async fn logout(jar: CookieJar) -> Result<impl IntoResponse> {
    match jar.get("token") {
        Some(token) => {
            Redis::del_session(token.value()).await?;
            ok(()).map(|r| (jar.remove(Cookie::build(("token", "")).path("/")), r))
        }
        None => Err(Error::new(401, "require to login")),
    }
}

struct Redis;

impl Redis {
    fn key(key: &str) -> String {
        format!("auth:{key}")
    }

    async fn set_session(session: &str) -> Result<String> {
        let redis = redis().await?;
        let mut con = redis.get_multiplexed_async_connection().await?;

        let key = ulid::Ulid::new().to_string();
        con.set_ex(
            Self::key(&key),
            session,
            (SESSION_EXPIRE * 86_400 + 1_200) as u64,
        )
        .await?;

        Ok(key)
    }

    async fn del_session(key: &str) -> Result<()> {
        let redis = redis().await?;
        let mut con = redis.get_multiplexed_async_connection().await?;
        con.del(Self::key(key)).await?;
        Ok(())
    }
}

pub fn router() -> Router {
    Router::new()
        .route("/", get(login_page))
        .route("/login", get(login_page))
        .route("/register", get(register_page))
        .route("/logout", post(logout))
        .nest(
            "/api",
            Router::new()
                .route("/register", post(register))
                .route("/login", post(login)),
        )
}
