use serde::Deserialize;
use twitch_oauth2::{ClientId, ClientSecret};

#[derive(Debug, Clone, Deserialize)]
pub struct BotConfig {
    pub client_id: ClientId,
    pub client_secret: ClientSecret,
    pub bot_user_id: String,
    pub streamer_user_id: String,
}

impl BotConfig {
    pub(super) fn validate(&self, key: &str) -> anyhow::Result<()> {
        if self.client_id.as_str().is_empty() {
            anyhow::bail!("[twitch_bot.{key}]: client_id must not be empty");
        }
        if self.client_secret.as_str().is_empty() {
            anyhow::bail!("[twitch_bot.{key}]: client_secret must not be empty");
        }

        if self.bot_user_id.is_empty() {
            anyhow::bail!("[twitch_bot.{key}]: bot_user_id must not be empty");
        }

        if self.streamer_user_id.is_empty() {
            anyhow::bail!("[twitch_bot.{key}]: streamer_user_id must not be empty");
        }

        Ok(())
    }
}
