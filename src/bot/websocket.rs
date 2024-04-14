use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::Arc;

use anyhow::Context;
use dashmap::DashMap;
use tokio_tungstenite::tungstenite;
use tracing::Instrument;
use twitch_api::eventsub::channel::chat::message::ChannelChatMessageV1Payload;
use twitch_api::twitch_oauth2::{TwitchToken, UserToken};
use twitch_api::types::UserId;
use twitch_api::{
    eventsub::{
        self,
        event::websocket::{EventsubWebsocketData, ReconnectPayload, SessionData, WelcomePayload},
        Event,
    },
    types::{self},
    HelixClient,
};
use twitch_oauth2::AppAccessToken;

use crate::Events;

pub struct WebsocketClient {
    /// The session id of the websocket connection
    pub session_id: Option<String>,
    /// The token used to authenticate with the Twitch API
    pub token: AppAccessToken,
    pub bot_user_id: UserId,
    /// The client used to make requests to the Twitch API
    pub client: HelixClient<'static, reqwest::Client>,
    /// The user id of the channel we want to listen to
    pub user_id: UserId,
    /// The url to use for websocket
    pub connect_url: url::Url,

    pub events: Arc<Events>,

    pub on_ready_sender: Option<tokio::sync::mpsc::Sender<String>>,
}

impl WebsocketClient {
    /// Connect to the websocket and return the stream
    pub async fn connect(
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

    pub fn start(
        mut self,
    ) -> anyhow::Result<(
        tokio::task::JoinHandle<anyhow::Result<()>>,
        tokio::sync::mpsc::Receiver<String>,
    )> {
        let (sender, receiver) = tokio::sync::mpsc::channel(100);

        self.on_ready_sender = Some(sender);
        let join_handle = tokio::spawn(async move { self.run().await });

        Ok((join_handle, receiver))
    }

    /// Run the websocket subscriber
    // #[tracing::instrument(name = "subscriber", skip_all, fields())]
    pub async fn run(mut self) -> anyhow::Result<()> {
        // Establish the stream
        let mut s = self
            .connect()
            .await
            .context("when establishing connection")?;
        // Loop over the stream, processing messages as they come in.
        loop {
            tokio::select!(
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
            })
        }
    }

    /// Process a message from the websocket
    pub async fn process_message(&mut self, msg: tungstenite::Message) -> anyhow::Result<()> {
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

    pub async fn process_welcome_message(&mut self, data: SessionData<'_>) -> anyhow::Result<()> {
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
            anyhow::bail!("Token is elasped :(");
            // self.token =
            //     crate::util::get_access_token(self.client.get_client(), &self.opts).await?;
        }
        Ok(())
    }
}