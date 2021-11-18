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
            .author(crate_authors!(", "))
            .about(crate_description!())
            .arg(
                Arg::with_name("config")
                    .short("c")
                    .value_name("FILE")
                    .help("Set a custom config file to read")
                    .takes_value(true),
            )
            .get_matches();

        let config_path = PathBuf::from(
            matches
                .value_of_os("config")
                .unwrap_or(OsStr::new("config.toml")),
        );

        Args { config_path }
    }
}
