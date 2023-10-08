use crate::args::Args;
use crate::config::Config;
use crate::migration::Migrator;
use anyhow::anyhow;
use anyhow::Context;
use clap::Parser;
use futures::future::FusedFuture;
use futures::FutureExt;
use lazy_static::lazy_static;
use sea_orm::Database;
use sea_orm_migration::MigratorTrait;
use std::process::ExitCode;
use tokio_util::sync::CancellationToken;

pub mod api;
pub mod args;
pub mod config;
pub mod migration;
pub mod models;
pub mod shutdown;
pub mod web;

lazy_static! {
    static ref HTTP_CLIENT: reqwest::Client = reqwest::Client::new();
}

#[tokio::main]
async fn main() -> ExitCode {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    // This is done to print the error from main to tracing. It's possible to directly return errors from main,
    // but they are printed to stdout, and not the logger.
    match main_inner().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!("{:#}", e);
            ExitCode::FAILURE
        }
    }
}

async fn main_inner() -> anyhow::Result<()> {
    let args = Args::parse();
    tracing::debug!("Parsed args as {:?}", args);

    let config = Box::leak(Box::new(
        Config::load(&args.config_path)
            .await
            .context("Failed to load config")?,
    ));
    tracing::debug!("Successfully loaded config: {:#?}", config);

    // db init
    let db = Box::leak(Box::new(
        Database::connect(&config.database)
            .await
            .context("Failed to connect to database")?,
    ));
    Migrator::up(&(*db), None)
        .await
        .context("Failed to run database migrations")?;
    tracing::info!("Successfully ran database migrations");

    let shutdown_signal = CancellationToken::new();

    let webserver = web::run(config, db, shutdown_signal.clone())
        .await
        .context("Failed to run web server")?;
    let mut webserver_join_handle = tokio::spawn(webserver).fuse();

    let os_shutdown_signal = shutdown::shutdown_signal().fuse();
    futures::pin_mut!(os_shutdown_signal);

    let mut result: anyhow::Result<()> = Ok(());
    loop {
        if webserver_join_handle.is_terminated() {
            tracing::info!("Everything shut down successfully, ending");
            break;
        }

        tokio::select! {
            _ = &mut os_shutdown_signal, if !os_shutdown_signal.is_terminated() => {
                tracing::debug!("Received shutdown signal from operating system, shutting down application...");
                shutdown_signal.cancel();
            },
            webserver_result = (&mut webserver_join_handle), if !webserver_join_handle.is_terminated() => {
                // two cases:
                // - webserver ends on its own WITHOUT us sending the
                //   shutdown signal first (fatal error probably)
                //   os_shutdown_signal.is_terminated() will be FALSE
                // - webserver ends after Ctrl-C shutdown request
                //   os_shutdown_signal.is_terminated() will be TRUE
                result = match webserver_result {
                    Ok(Ok(())) => {
                        if !shutdown_signal.is_cancelled() {
                            shutdown_signal.cancel();
                            Err(anyhow!("Webserver ended without error even though no shutdown was requested"))
                        } else {
                            // regular end after graceful shutdown request
                            tracing::debug!("Webserver has successfully shut down gracefully");
                            Ok(())
                        }
                    },
                    Ok(Err(tower_error)) => {
                        shutdown_signal.cancel();
                        Err(tower_error).context("Webserver encountered fatal error")
                    },
                    Err(join_error) => {
                        shutdown_signal.cancel();
                        Err(join_error).context("Webserver tokio task ended abnormally")
                    }
                }
            }
        }
    }

    result
}
