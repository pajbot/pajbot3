use async_trait::async_trait;
use chrono::{DateTime, Utc};
use redis::AsyncCommands;
use tokio_util::sync::CancellationToken;
use tracing::info;
use twitch_irc::login::{RefreshingLoginCredentials, TokenStorage, UserAccessToken};
use twitch_irc::{ClientConfig, SecureTCPTransport, TwitchIRCClient};

#[derive(Debug)]
struct PB1TokenStorage {
    bot_id: String,
}

#[derive(serde::Deserialize)]
pub struct Pajbot1UserAccessToken {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: Option<f64>,
    pub created_at: Option<f64>,
}

#[async_trait]
impl TokenStorage for PB1TokenStorage {
    type LoadError = anyhow::Error; // or some other error
    type UpdateError = std::io::Error;

    async fn load_token(&mut self) -> Result<UserAccessToken, Self::LoadError> {
        info!("load token called");

        let client = redis::Client::open("redis://127.0.0.1").unwrap();
        let mut con = client.get_async_connection().await.unwrap();

        let token: String = con
            .get(format!("authentication:user-access-token:{}", self.bot_id))
            .await
            .unwrap();

        let token: Pajbot1UserAccessToken = serde_json::from_str(token.as_str()).unwrap();

        let created_at = (token.created_at.unwrap() / 1000.0) as i64;
        let created_at = DateTime::from_timestamp(created_at, 0)
            .ok_or_else(|| anyhow::anyhow!("missing created_at from redis token"))?;

        let expires_in = (token.expires_in.unwrap() / 1000.0) as i64;

        let token = UserAccessToken {
            access_token: token.access_token,
            refresh_token: token.refresh_token,
            created_at,
            expires_at: Some(created_at + chrono::Duration::seconds(expires_in)),
        };

        Ok(token)
    }

    async fn update_token(&mut self, token: &UserAccessToken) -> Result<(), Self::UpdateError> {
        // Called after the token was updated successfully, to save the new token.
        // After `update_token()` completes, the `load_token()` method should then return
        // that token for future invocations
        todo!()
    }
}

// TwitchBot is a user connected to a channel
pub struct TwitchBot {
    client: TwitchIRCClient<SecureTCPTransport, RefreshingLoginCredentials<PB1TokenStorage>>,
}

pub struct Bot {
    clients: Vec<TwitchIRCClient<SecureTCPTransport, RefreshingLoginCredentials<PB1TokenStorage>>>,
}

impl Bot {
    fn new() -> Self {
        Bot {
            clients: Vec::new(),
        }
    }

    fn add_client(
        &mut self,
        config: &crate::config::BotConfig,
        shutdown_signal: CancellationToken,
    ) -> anyhow::Result<()> {
        // these credentials can be generated for your app at https://dev.twitch.tv/console/apps
        // the bot's username will be fetched based on your access token
        let storage = PB1TokenStorage {
            bot_id: config.bot_id.clone(),
        };

        let credentials = RefreshingLoginCredentials::init(
            config.client_id.clone(),
            config.client_secret.clone(),
            storage,
        );
        // It is also possible to use the same credentials in other places
        // such as API calls by cloning them.
        let config = ClientConfig::new_simple(credentials);
        let (mut incoming_messages, client) = TwitchIRCClient::<SecureTCPTransport, _>::new(config);

        client.join("pajlada".to_owned())?;

        let twitch_bot = TwitchBot { client };

        tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                println!("Received message: {:?}", message);
            }
            info!("bot handle quit bye bye");
        });

        tokio::spawn(async move {
            let _twitch_bot = twitch_bot;
            shutdown_signal.cancelled().await;
        });

        Ok(())
    }
}

pub async fn run(config: &crate::Config, shutdown_signal: CancellationToken) -> anyhow::Result<()> {
    let mut bot = Bot::new();

    for (bot_id, bot_config) in &config.bot {
        info!("Loading bot with id {bot_id}");
        bot.add_client(bot_config, shutdown_signal.clone())?;
    }

    Ok(())
}
