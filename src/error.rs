use atlas::http_api::{AsError, Result as AtlasResult};
use axum::{
    Json,
    http::status::InvalidStatusCode,
    response::{IntoResponse, Response},
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;
pub type R<T> = AtlasResult<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("code: {0}, message: {1}")]
    Message(u32, String),

    #[error(transparent)]
    Sqlx(#[from] sqlx::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    InvalidStatusCode(#[from] InvalidStatusCode),

    #[error(transparent)]
    Http(#[from] axum::http::Error),

    #[error(transparent)]
    StdIo(#[from] std::io::Error),

    #[error(transparent)]
    AddrParse(#[from] std::net::AddrParseError),

    #[error(transparent)]
    Toml(#[from] toml::de::Error),

    #[error(transparent)]
    Redis(#[from] redis::RedisError),

    #[error(transparent)]
    Argon2Phc(#[from] argon2::password_hash::phc::Error),

    #[error(transparent)]
    Argon2(#[from] argon2::password_hash::Error),
}

impl Error {
    pub fn new(code: u32, message: &str) -> Self {
        Error::Message(code, message.to_string())
    }

    fn code(&self) -> u32 {
        match self {
            Self::Message(code, _) => *code,
            _ => 500,
        }
    }

    fn message(&self) -> String {
        match self {
            Self::Message(_, message) => message.clone(),
            _ => self.to_string(),
        }
    }
}

impl<M: std::fmt::Display> AsError<M> for Error {
    fn from_code_message(code: u32, message: M) -> Self {
        Error::Message(code, message.to_string())
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        (
            axum::http::StatusCode::OK,
            Json(serde_json::json!({ "code": self.code(), "message": self.message() })),
        )
            .into_response()
    }
}
