use std::sync::Arc;

use crate::config::BotConfig;
use crate::{Config, Events};
use anyhow::Context;
use sea_orm::DatabaseConnection;
use tokio_util::sync::CancellationToken;
use twitch_api::client::ClientDefault;
use twitch_api::twitch_oauth2::{AccessToken, UserToken};
use twitch_api::HelixClient;
use twitch_oauth2::AppAccessToken;
use twitch_types::UserId;

mod websocket;

pub async fn run(
    config: &'static Config,
    bot_config: &'static BotConfig,
    db: &'static DatabaseConnection,
    events: Arc<Events>,
    shutdown_signal: CancellationToken,
) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
    let client: HelixClient<_> = twitch_api::HelixClient::with_client(
        <reqwest::Client>::default_client_with_name(Some(
            "pajbot/3.0"
                .parse()
                .with_context(|| "when creating header name")
                .unwrap(),
        ))
        .with_context(|| "when creating client")?,
    );

    let token = AppAccessToken::get_app_access_token(
        client.get_client(),
        bot_config.client_id.clone(),
        bot_config.client_secret.clone(),
        vec![],
    )
    .await?;

    let conduits = client.get_conduits(&token).await?;
    println!("Conduits: {conduits:?}");

    let first_conduit = conduits.first().unwrap().clone();

    let transport = twitch_api::eventsub::Transport::conduit(first_conduit.id.clone());
    let bot_user_id: UserId = bot_config.bot_user_id.clone().into();
    let streamer_user_id: UserId = bot_config.streamer_user_id.clone().into();

    match client
        .create_eventsub_subscription(
            twitch_api::eventsub::channel::ChannelChatMessageV1::new(
                streamer_user_id.clone(),
                bot_user_id.clone(),
            ),
            transport.clone(),
            &token,
        )
        .await
    {
        Ok(created_subscription) => {
            tracing::info!("Created subscription: {created_subscription:?}");
        }
        Err(e) => match e {
            twitch_api::helix::ClientRequestError::HelixRequestPostError(e) => match e {
                twitch_api::helix::HelixRequestPostError::Error {
                    ref error,
                    status,
                    ref message,
                    ref uri,
                    ref body,
                } => {
                    if status == reqwest::StatusCode::CONFLICT {
                        tracing::info!("This subscription already exists!");
                    } else {
                        return Err(e.into());
                    }
                }
                e => {
                    return Err(e.into());
                }
            },
            e => {
                return Err(e.into());
            }
        },
    }

    let websocket_client = websocket::WebsocketClient {
        session_id: None,
        token: token.clone(),
        bot_user_id: bot_config.bot_user_id.clone().into(),
        client: client.clone(),
        connect_url: twitch_api::TWITCH_EVENTSUB_WEBSOCKET_URL.clone(),
        events,
        on_ready_sender: None,
    };

    let (join_handle, mut recv) = websocket_client.start(shutdown_signal)?;

    tokio::spawn(async move {
        while let Some(xd) = recv.recv().await {
            let shard = twitch_api::eventsub::Shard::new(
                "1",
                twitch_api::eventsub::Transport::websocket(xd),
            );

            let response = client
                .update_conduit_shards(&first_conduit.id, vec![shard], &token)
                .await;
            tracing::info!("response: {response:?}");
            // can do helix subscriptions here
            // let subscription = twitch_api::eventsub::channel::ChannelMes

            // };
            // let res = client
            //     .create_eventsub_subscription(subscription, transport, &token)
            //     .await
            //     .unwrap();
        }
    });

    Ok(join_handle)
}
