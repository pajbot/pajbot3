pub mod database;
pub mod web;

use crate::api::twitch;
use crate::config::database::DatabaseConfig;
use crate::config::web::WebConfig;
use anyhow::Context;
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    #[serde(default)]
    pub web: WebConfig,
    pub twitch_api: twitch::ApiClientCredentials,
}

impl Config {
    pub async fn load(path: &Path) -> anyhow::Result<Config> {
        let file_contents = tokio::fs::read(path)
            .await
            .with_context(|| format!("Failed to read Config file from {}", path.display()))?;
        let config = toml::from_str(
            &String::from_utf8(file_contents).context("Config file contains non-UTF8 text")?,
        )
        .context("Failed to parse Config file contents")?;
        Ok(config)
    }
}
