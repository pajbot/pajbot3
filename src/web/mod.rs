pub mod auth;
pub mod error;

use crate::config::web::ListenAddr;
use crate::db::DataStorage;
use crate::web::error::ApiError;
use crate::Config;
use axum::http::{header, Method};
use axum::routing::get;
use axum::routing::post;
use axum::Router;
use futures::future::BoxFuture;
use std::net::SocketAddr;
use thiserror::Error;
use tokio_util::sync::CancellationToken;
use tower_http::cors::{self, CorsLayer};
#[cfg(unix)]
use {
    hyperlocal::UnixServerExt, std::fs::Permissions, std::os::unix::fs::PermissionsExt,
    std::path::Path,
};

#[derive(Clone, Copy)]
pub struct WebAppData {
    config: &'static Config,
    db: &'static DataStorage,
}

#[derive(Error, Debug)]
pub enum BindError {
    #[error("Failed to bind to address `{0}`: {1}")]
    BindTcp(&'static SocketAddr, hyper::Error),
    #[cfg(unix)]
    #[error("Failed to bind to unix socket `{}`: {1}", .0.display())]
    BindUnix(&'static Path, std::io::Error),
    #[cfg(unix)]
    #[error("Failed to alter permissions on unix socket `{}` to `{1:?}`: {2}", .0.display())]
    SetPermissions(&'static Path, Permissions, std::io::Error),
}

pub async fn run(
    config: &'static Config,
    db: &'static DataStorage,
    shutdown_signal: CancellationToken,
) -> Result<BoxFuture<'static, hyper::Result<()>>, BindError> {
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

    Ok(match &config.web.listen_address {
        ListenAddr::Tcp { address } => Box::pin(
            axum::Server::try_bind(address)
                .map_err(|e| BindError::BindTcp(address, e))?
                .serve(app.into_make_service())
                .with_graceful_shutdown(async move {
                    shutdown_signal.cancelled().await;
                }),
        ),
        #[cfg(unix)]
        ListenAddr::Unix { path } => {
            let builder =
                axum::Server::bind_unix(path).map_err(|e| BindError::BindUnix(path, e))?;
            let permissions = Permissions::from_mode(0o777);
            tokio::fs::set_permissions(path, permissions.clone())
                .await
                .map_err(|e| BindError::SetPermissions(path, permissions, e))?;
            Box::pin(
                builder
                    .serve(app.into_make_service())
                    .with_graceful_shutdown(async move {
                        shutdown_signal.cancelled().await;
                    }),
            )
        }
    })
}
