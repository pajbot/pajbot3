use clap::{crate_name, crate_version, App, Arg};
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    pub config_path: PathBuf,
}

impl Args {
    pub fn parse() -> Args {
        let matches = App::new(crate_name!())
            .version(crate_version!())
            .arg(
                Arg::new("config")
                    .short('c')
                    .long("config")
                    .value_name("FILE")
                    .help("File path to read config from")
                    .default_value_os(OsStr::new("config.toml"))
                    .takes_value(true),
            )
            .get_matches();

        let config_path = PathBuf::from(matches.value_of_os("config").unwrap());

        Args { config_path }
    }
}
