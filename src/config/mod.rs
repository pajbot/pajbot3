pub mod database;
pub mod web;

use crate::api::twitch;
use crate::config::database::DatabaseConfig;
use crate::config::web::WebConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct BotConfig {
    pub client_id: String,
    pub client_secret: String,
    pub db_schema: String,
    pub bot_id: String,
    pub target_id: String,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub web: WebConfig,
    pub twitch_api: twitch::ApiClientCredentials,

    pub bot: HashMap<String, BotConfig>,
}

impl Config {
    pub async fn load(path: &Path) -> Result<Config, LoadConfigError> {
        let file_contents = tokio::fs::read_to_string(path)
            .await
            .map_err(LoadConfigError::ReadFile)?;
        let config = toml::from_str(&file_contents).map_err(LoadConfigError::ParseContents)?;
        Ok(config)
    }
}

#[derive(Error, Debug)]
pub enum LoadConfigError {
    #[error("Failed to read file: {0}")]
    ReadFile(std::io::Error),
    #[error("Failed to parse contents: {0}")]
    ParseContents(toml::de::Error),
}
