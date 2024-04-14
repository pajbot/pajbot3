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
    // let token_from_env = std::env::var("PB3_ACCESS_TOKEN")?;
    // let token = UserToken::from_token(&client, AccessToken::from(token_from_env)).await?;
    let token = AppAccessToken::get_app_access_token(
        client.get_client(),
        bot_config.client_id.clone(),
        bot_config.client_secret.clone(),
        vec![],
    )
    .await?;

    let user_id = "11148817";

    // let resulting_conduit = client.create_conduit(2, &token).await?;
    // println!("Created a conduit: {resulting_conduit:?}");

    let conduits = client.get_conduits(&token).await?;
    println!("Conduits: {conduits:?}");

    let first_conduit = conduits.first().unwrap().clone();

    let transport = twitch_api::eventsub::Transport::conduit(first_conduit.id.clone());
    let pajlada_user_id: UserId = user_id.clone().into();
    let bot_user_id: UserId = bot_config.bot_user_id.clone().into();
    let res = client
        .create_eventsub_subscription(
            twitch_api::eventsub::channel::ChannelChatMessageV1::new(
                // self.user_id.clone(),
                pajlada_user_id.clone(),
                bot_user_id.clone(),
            ),
            transport.clone(),
            &token,
        )
        .await;

    match res {
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
                        panic!("Unhandled error creating sbuscription: {e:?}");
                    }
                }
                e => panic!("Ran into an unhandled error creating the subscription: {e:?}"),
            },
            e => panic!("Ran into an unknown error creating the subscription: {e:?}"),
        },
    }

    let websocket_client = websocket::WebsocketClient {
        session_id: None,
        token: token.clone(),
        bot_user_id: bot_config.bot_user_id.clone().into(),
        client: client.clone(),
        user_id: user_id.into(),
        connect_url: twitch_api::TWITCH_EVENTSUB_WEBSOCKET_URL.clone(),
        events,
        on_ready_sender: None,
    };

    let (join_handle, mut recv) = websocket_client.start()?;

    tokio::spawn(async move {
        while let Some(xd) = recv.recv().await {
            tracing::info!("received {xd}");

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
