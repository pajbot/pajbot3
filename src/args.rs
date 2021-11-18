use clap::{crate_authors, crate_description, crate_name, crate_version, App, Arg};
use std::ffi::OsStr;
use std::path::PathBuf;

#[derive(Debug)]
pub struct Args {
    config_path: PathBuf,
}

impl Args {
    pub fn parse() -> Args {
        let matches = App::new(crate_name!())
            .version(crate_version!())
            .arg(
                Arg::with_name("config")
                    .short("c")
                    .long("config")
                    .value_name("FILE")
                    .help("Set a custom config file to read")
                    .default_value_os(OsStr::new("config.toml"))
                    .takes_value(true),
            )
            .get_matches();

        let config_path = PathBuf::from(matches.value_of_os("config").unwrap());

        Args { config_path }
    }
}
