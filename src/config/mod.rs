mod bot;
pub mod database;
pub mod web;

use crate::api::twitch;
use crate::config::database::DatabaseConfig;
use crate::config::web::WebConfig;
use anyhow::Context;
pub use bot::BotConfig;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub database: DatabaseConfig,
    #[serde(default)]
    pub web: WebConfig,
    pub twitch_api: twitch::ApiClientCredentials,
    #[serde(default)]
    pub twitch_bot: HashMap<String, BotConfig>,
}

impl Config {
    pub async fn load(path: &Path) -> anyhow::Result<Self> {
        let file_contents = tokio::fs::read(path)
            .await
            .with_context(|| format!("Failed to read Config file from {}", path.display()))?;
        let config: Self = toml::from_str(
            &String::from_utf8(file_contents).context("Config file contains non-UTF8 text")?,
        )
        .context("Failed to parse Config file contents")?;

        config.validate()?;

        Ok(config)
    }

    fn validate(&self) -> anyhow::Result<()> {
        if self.twitch_bot.is_empty() {
            anyhow::bail!("You must specify at least one twitch_bot section. Check the example config file for how it should be structured.")
        }

        for (config_key, twitch_bot_config) in &self.twitch_bot {
            twitch_bot_config.validate(config_key)?;
        }

        Ok(())
    }
}
