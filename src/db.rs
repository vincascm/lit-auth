use chrono::NaiveDateTime;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, query, query_as};

use crate::{error::Result, statics::db};

#[derive(Debug, FromRow, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub created_at: NaiveDateTime,
}

impl User {
    pub async fn create(username: &str, password_hash: &str) -> Result<i64> {
        let res = query("INSERT INTO users (username, password_hash) VALUES (?, ?)")
            .bind(username)
            .bind(password_hash)
            .execute(db().await?)
            .await?;
        Ok(res.last_insert_rowid())
    }

    pub async fn find_by_username(username: &str) -> Result<Option<Self>> {
        Ok(
            query_as(
                "SELECT id, username, password_hash, created_at FROM users WHERE username = ?",
            )
            .bind(username)
            .fetch_optional(db().await?)
            .await?,
        )
    }
}
