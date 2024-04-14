use std::sync::Arc;

use anyhow::Context;
use tokio_tungstenite::tungstenite;
use tokio_util::sync::CancellationToken;
use twitch_api::twitch_oauth2::TwitchToken;
use twitch_api::{
    eventsub::{
        self,
        event::websocket::{EventsubWebsocketData, ReconnectPayload, SessionData, WelcomePayload},
        Event,
    },
    HelixClient,
};
use twitch_oauth2::AppAccessToken;

use crate::config::BotConfig;
use crate::Events;

pub struct WebsocketClient {
    /// The session id of the websocket connection
    session_id: Option<String>,
    /// The token used to authenticate with the Twitch API
    token: AppAccessToken,
    /// The client used to make requests to the Twitch API
    client: HelixClient<'static, reqwest::Client>,
    bot_config: &'static BotConfig,
    /// The url to use for websocket
    connect_url: url::Url,

    events: Arc<Events>,

    on_ready_sender: Option<tokio::sync::mpsc::Sender<String>>,
}

impl WebsocketClient {
    pub fn new(
        token: AppAccessToken,
        client: HelixClient<'static, reqwest::Client>,
        bot_config: &'static BotConfig,
        connect_url: url::Url,
        events: Arc<Events>,
    ) -> Self {
        Self {
            session_id: None,
            token,
            client,
            bot_config,
            connect_url,
            events,
            on_ready_sender: None,
        }
    }

    pub fn start(
        mut self,
        shutdown_signal: CancellationToken,
    ) -> anyhow::Result<(
        tokio::task::JoinHandle<anyhow::Result<()>>,
        tokio::sync::mpsc::Receiver<String>,
    )> {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        self.on_ready_sender = Some(sender);
        let join_handle = tokio::spawn(async move { self.run(shutdown_signal).await });

        Ok((join_handle, receiver))
    }

    /// Connect to the websocket and return the stream
    async fn connect(
        &self,
    ) -> anyhow::Result<
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    > {
        tracing::info!("connecting to twitch");
        let config = tungstenite::protocol::WebSocketConfig {
            // max_write_buffer_size: 2048,
            max_message_size: Some(64 << 20), // 64 MiB
            max_frame_size: Some(16 << 20),   // 16 MiB
            accept_unmasked_frames: false,
            ..tungstenite::protocol::WebSocketConfig::default()
        };
        let (socket, _) =
            tokio_tungstenite::connect_async_with_config(&self.connect_url, Some(config), false)
                .await
                .context("Can't connect")?;

        Ok(socket)
    }

    /// Run the websocket subscriber
    // #[tracing::instrument(name = "subscriber", skip_all, fields())]
    async fn run(mut self, shutdown_signal: CancellationToken) -> anyhow::Result<()> {
        // Establish the stream
        let mut s = self
            .connect()
            .await
            .context("when establishing connection")?;
        // Loop over the stream, processing messages as they come in.
        loop {
            tokio::select! {
                _ = shutdown_signal.cancelled() => {
                    tracing::info!("Shutdown signal fired!!!!!!");
                    break Ok(());
                }
                Some(msg) = futures::StreamExt::next(&mut s) => {
                    let msg = match msg {
                        Err(tungstenite::Error::Protocol(
                            tungstenite::error::ProtocolError::ResetWithoutClosingHandshake,
                        )) => {
                            tracing::warn!(
                                "connection was sent an unexpected frame or was reset, reestablishing it"
                            );
                            s = self
                                .connect()
                                .await
                                .context("when reestablishing connection")?;
                            continue
                        }
                        _ => msg.context("when getting message")?,
                    };
                    self.process_message(msg).await?
                }
            }
        }
    }

    /// Process a message from the websocket
    async fn process_message(&mut self, msg: tungstenite::Message) -> anyhow::Result<()> {
        match msg {
            tungstenite::Message::Text(s) => {
                // Parse the message into a [twitch_api::eventsub::EventsubWebsocketData]
                match Event::parse_websocket(&s)? {
                    EventsubWebsocketData::Welcome {
                        payload: WelcomePayload { session },
                        ..
                    }
                    | EventsubWebsocketData::Reconnect {
                        payload: ReconnectPayload { session },
                        ..
                    } => {
                        self.process_welcome_message(session).await?;
                        Ok(())
                    }
                    // Here is where you would handle the events you want to listen to
                    EventsubWebsocketData::Notification {
                        metadata: _,
                        payload,
                    } => {
                        match payload {
                            Event::ChannelBanV1(eventsub::Payload { message, .. }) => {
                                tracing::info!(?message, "got ban event");
                            }
                            Event::ChannelUnbanV1(eventsub::Payload { message, .. }) => {
                                tracing::info!(?message, "got ban event");
                            }
                            Event::ChannelChatMessageV1(eventsub::Payload { message, .. }) => {
                                match message {
                                    eventsub::Message::VerificationRequest(_) => unreachable!(
                                        "This should only be reachable by webhook events"
                                    ),
                                    eventsub::Message::Revocation() => unreachable!(
                                        "This should be handled by the Revocation portion below"
                                    ),
                                    eventsub::Message::Notification(message) => {
                                        self.events.publish_chat_message(message)?;
                                    }
                                    _ => panic!("non_exhaustive enum {message:?}"),
                                }
                            }
                            _ => {}
                        }
                        Ok(())
                    }
                    EventsubWebsocketData::Revocation {
                        metadata,
                        payload: _,
                    } => anyhow::bail!("got revocation event: {metadata:?}"),
                    EventsubWebsocketData::Keepalive {
                        metadata: _,
                        payload: _,
                    } => Ok(()),
                    _ => Ok(()),
                }
            }
            tungstenite::Message::Close(_) => todo!(),
            _ => Ok(()),
        }
    }

    async fn process_welcome_message(&mut self, data: SessionData<'_>) -> anyhow::Result<()> {
        self.session_id = Some(data.id.to_string());
        tracing::info!("Processing welcome message");
        self.on_ready_sender
            .as_mut()
            .unwrap()
            .send(data.id.to_string())
            .await?;
        tracing::info!("Sent to on ready sender");
        if let Some(url) = data.reconnect_url {
            self.connect_url = url.parse()?;
        }
        // check if the token is expired, if it is, request a new token. This only works if using a oauth service for getting a token
        if self.token.is_elapsed() {
            tracing::info!("Refreshing WebSocket AppAccessToken");
            self.token = AppAccessToken::get_app_access_token(
                self.client.get_client(),
                self.bot_config.client_id.clone(),
                self.bot_config.client_secret.clone(),
                vec![],
            )
            .await?;
        }
        Ok(())
    }
}
