use serde::Deserialize;

use crate::error::Result;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub listen_addr: String,
    pub database_url: String,
    pub redis_url: String,
    pub allow_register: bool,
}

impl Config {
    pub fn load() -> Result<Config> {
        let filename = std::env::args()
            .nth(1)
            .unwrap_or_else(|| "config.toml".to_owned());
        Ok(toml::from_str(&std::fs::read_to_string(filename)?)?)
    }
}
