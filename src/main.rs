use crate::args::Args;
use crate::config::Config;
use std::process;

mod args;
mod config;

#[tokio::main]
async fn main() {
    // install global collector configured based on RUST_LOG env var.
    tracing_subscriber::fmt::init();

    let args = Args::parse();
    tracing::debug!("Parsed args as {:?}", args);

    let config = match Config::load(&args.config_path).await {
        Ok(config) => config,
        Err(e) => {
            tracing::error!("Failed to load config: {}", e);
            process::exit(1);
        }
    };
    tracing::debug!("Successfully loaded config: {:?}", config);
}
