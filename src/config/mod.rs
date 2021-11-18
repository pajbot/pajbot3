mod database;

use std::path::Path;
use crate::config::database::DatabaseConfig;
use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig
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
