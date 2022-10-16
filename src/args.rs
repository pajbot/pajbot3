use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long = "config", value_name = "FILE")]
    pub config_path: PathBuf,
}
