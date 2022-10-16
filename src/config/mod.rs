pub mod database;
pub mod web;

use crate::api::twitch;
use crate::config::database::DatabaseConfig;
use crate::config::web::WebConfig;
use serde::Deserialize;
use std::path::Path;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub database: DatabaseConfig,
    #[serde(default)]
    pub web: WebConfig,
    pub twitch_api: twitch::ApiClientCredentials,
}

impl Config {
    pub async fn load(path: &Path) -> Result<Config, LoadConfigError> {
        let file_contents = tokio::fs::read(path)
            .await
            .map_err(LoadConfigError::ReadFile)?;
        let config = toml::from_slice(&file_contents).map_err(LoadConfigError::ParseContents)?;
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
