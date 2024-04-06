use crate::Config;
use sea_orm::DatabaseConnection;
use tokio_util::sync::CancellationToken;
use twitch_api::twitch_oauth2::{AccessToken, UserToken};
use twitch_api::HelixClient;

mod websocket;

pub async fn run(
    config: &'static Config,
    db: &'static DatabaseConnection,
    shutdown_signal: CancellationToken,
) -> anyhow::Result<tokio::task::JoinHandle<anyhow::Result<()>>> {
    /*
    let client: HelixClient<_> = twitch_api::HelixClient::with_client(
        <reqwest::Client>::default_client_with_name(Some(
            "twitch-rs/eventsub"
                .parse()
                .wrap_err_with(|| "when creating header name")
                .unwrap(),
        ))
        .wrap_err_with(|| "when creating client")?,
    );
    let token_from_env = std::env::var("PB3_ACCESS_TOKEN")?;
    let token = UserToken::from_token(&client, AccessToken::from(token_from_env)).await?;

    let user_id = "11148817";

    println!(
        "Channel: {:?}",
        client.get_channel_from_login("twitchdev", &token).await?
    );

    let websocket_client = websocket::WebsocketClient {
        session_id: None,
        token: token.clone(),
        client: client.clone(),
        user_id: user_id.into(),
        connect_url: twitch_api::TWITCH_EVENTSUB_WEBSOCKET_URL.clone(),
        on_ready_sender: None,
    };

    let (join_handle, mut recv) = websocket_client.start()?;

    tokio::spawn(async move {
        while let Some(xd) = recv.recv().await {
            tracing::info!("received {xd}");
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
    */
    anyhow::bail!("asd")
}
