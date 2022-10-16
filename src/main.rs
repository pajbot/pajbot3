use crate::args::Args;
use crate::config::Config;
use futures::future::FusedFuture;
use futures::FutureExt;
use lazy_static::lazy_static;
use std::process;
use tokio_util::sync::CancellationToken;

pub mod api;
pub mod args;
pub mod config;
pub mod db;
pub mod shutdown;
pub mod web;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    tracing::debug!("Parsed args as {:?}", args);

    let config = match Config::load(&args.config_path).await {
        Ok(config) => Box::leak(Box::new(config)),
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            process::exit(1);
        }
    };
    tracing::debug!("Successfully loaded config: {:#?}", config);

    // db init
    let data_storage = Box::leak(Box::new(db::connect_to_postgresql(&config).await));
    let migrations_result = data_storage.run_migrations().await;
    match migrations_result {
        Ok(()) => {
            tracing::info!("Successfully ran database migrations");
        }
        Err(e) => {
            tracing::error!("Failed to run database migrations: {}", e);
            std::process::exit(1);
        }
    }

    let shutdown_signal = CancellationToken::new();

    let webserver = match web::run(config, data_storage, shutdown_signal.clone()).await {
        Ok(webserver) => webserver,
        Err(bind_error) => {
            tracing::error!("{}", bind_error);
            std::process::exit(1);
        }
    };
    let mut webserver_join_handle = tokio::spawn(webserver).fuse();

    let os_shutdown_signal = shutdown::shutdown_signal().fuse();
    futures::pin_mut!(os_shutdown_signal);

    let mut exit_code: i32 = 0;
    loop {
        if webserver_join_handle.is_terminated() {
            tracing::info!("Everything shut down successfully, ending");
            break;
        }

        tokio::select! {
            _ = &mut os_shutdown_signal, if !os_shutdown_signal.is_terminated() => {
                tracing::debug!("Received shutdown signal");
                shutdown_signal.cancel();
            },
            webserver_result = (&mut webserver_join_handle), if !webserver_join_handle.is_terminated() => {
                // two cases:
                // - webserver ends on its own WITHOUT us sending the
                //   shutdown signal first (fatal error probably)
                //   ctrl_c_event.is_terminated() will be FALSE
                // - webserver ends after Ctrl-C shutdown request
                //   ctrl_c_event.is_terminated() will be TRUE
                match webserver_result {
                    Ok(Ok(())) => {
                        if !shutdown_signal.is_cancelled() {
                            tracing::error!("Webserver ended without error even though no shutdown was requested (shutting down other parts of application gracefully)");
                            shutdown_signal.cancel();
                            exit_code = 1;
                        } else {
                            // regular end after graceful shutdown request
                            tracing::info!("Webserver has successfully shut down gracefully");
                        }
                    },
                    Ok(Err(tower_error)) => {
                        tracing::error!("Webserver encountered fatal error (shutting down other parts of application gracefully): {}", tower_error);
                        shutdown_signal.cancel();
                        exit_code = 1;
                    },
                    Err(join_error) => {
                        tracing::error!("Webserver tokio task ended abnormally (shutting down other parts of application gracefully): {}", join_error);
                        shutdown_signal.cancel();
                        exit_code = 1;
                    }
                }
            }
        }
    }

    std::process::exit(exit_code);
}
