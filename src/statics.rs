use std::{str::FromStr, sync::OnceLock, time::Duration};

use sqlx::{
    SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqliteSynchronous},
};

use crate::{
    config::Config,
    error::{Error, Result},
};

static CONFIG: OnceLock<Config> = OnceLock::new();
static DB: OnceLock<SqlitePool> = OnceLock::new();
static REDIS: OnceLock<redis::Client> = OnceLock::new();

pub async fn init() -> Result<()> {
    let config = Config::load()?;
    let config = CONFIG.get_or_init(|| config);

    let options = SqliteConnectOptions::from_str(&config.database_url)?
        .journal_mode(SqliteJournalMode::Wal)
        .synchronous(SqliteSynchronous::Normal)
        .busy_timeout(Duration::from_secs(10));
    let db = SqlitePool::connect_with(options).await?;
    DB.get_or_init(|| db);

    let redis = redis::Client::open(config.redis_url.as_str())?;
    REDIS.get_or_init(|| redis);

    Ok(())
}

pub async fn cfg() -> Result<&'static Config> {
    CONFIG
        .get()
        .ok_or_else(|| Error::new(500, "get CONFIG error"))
}

pub async fn db() -> Result<&'static SqlitePool> {
    DB.get().ok_or_else(|| Error::new(500, "get db pool error"))
}

pub async fn redis() -> Result<&'static redis::Client> {
    REDIS
        .get()
        .ok_or_else(|| Error::new(500, "get redis error"))
}
