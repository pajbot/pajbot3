pub mod auth;
pub mod error;

use std::future::IntoFuture;

use crate::config::web::ListenAddr;
use crate::web::error::ApiError;
use crate::Config;
use anyhow::Context;
use axum::http::{header, Method};
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use futures::future::BoxFuture;
use sea_orm::DatabaseConnection;
use tokio::net::TcpListener;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{self, CorsLayer};

#[derive(Clone, Copy)]
pub struct WebAppData {
    config: &'static Config,
    db: &'static DatabaseConnection,
}

pub async fn run(
    config: &'static Config,
    db: &'static DatabaseConnection,
    shutdown_signal: CancellationToken,
) -> anyhow::Result<BoxFuture<'static, std::io::Result<()>>> {
    let shared_state = WebAppData { config, db };

    let cors = CorsLayer::new()
        .allow_methods(vec![Method::GET, Method::POST])
        .allow_headers(vec![
            header::AUTHORIZATION,
            header::ACCEPT,
            header::CONTENT_TYPE,
        ])
        .allow_origin(cors::Any); // TODO probably not any
    let method_fallback = || (|| async { ApiError::method_not_allowed() });
    let api = Router::new()
        .route(
            "/auth/create",
            post(auth::create_login::create_token).fallback(method_fallback()),
        )
        .route(
            "/auth/refresh",
            post(auth::refresh_login::refresh_token).fallback(method_fallback()),
        )
        .route(
            "/auth/revoke",
            post(auth::revoke_login::revoke_token).fallback(method_fallback()),
        )
        .layer(cors);

    let app = Router::new()
        .route("/", get(|| async { "Hello World!" }))
        .nest("/api/v1", api)
        .with_state(shared_state);

    Ok(match &config.web.listen {
        ListenAddr::Tcp { address } => {
            let listener = TcpListener::bind(address)
                .await
                .with_context(|| format!("Failed to bind to address `{}`", address))?;
            Box::pin(
                axum::serve(listener, app)
                    .with_graceful_shutdown(async move {
                        shutdown_signal.cancelled().await;
                    })
                    .into_future(),
            )
        }
        #[cfg(unix)]
        ListenAddr::Unix { .. } => {
            unimplemented!("Waiting for axum 0.8 for this to be done easily, see https://github.com/tokio-rs/axum/pull/2479");
        }
    })
}
