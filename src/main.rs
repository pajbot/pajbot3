#![deny(clippy::all)]
#![deny(clippy::cargo)]

use crate::args::Args;
use crate::config::Config;
use std::process;

mod args;
mod config;

#[tokio::main]
async fn main() {
    let args = Args::parse();
    dbg!(&args);

    let config = match Config::load(&args.config_path).await {
        Ok(config) => config,
        Err(e) => {
            // TODO use logger or something
            eprintln!("Failed to load config: {}", e);
            process::exit(1);
        }
    };
    dbg!(&config);
}
