use sea_orm::ConnectOptions;
use serde::Deserialize;
use serde_with::serde_as;
use serde_with::DeserializeAs;

/// Database config
#[serde_as]
#[derive(Debug, Clone, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    #[serde(default)]
    pub max_connections: Option<u32>,
    #[serde(default)]
    pub min_connections: Option<u32>,
    #[serde(default, with = "humantime_serde")]
    pub connect_timeout: Option<std::time::Duration>,
    #[serde(default, with = "humantime_serde")]
    pub idle_timeout: Option<std::time::Duration>,
    #[serde(default, with = "humantime_serde")]
    pub acquire_timeout: Option<std::time::Duration>,
    #[serde(default, with = "humantime_serde")]
    pub max_lifetime: Option<std::time::Duration>,
    #[serde_as(as = "Option<LogLevelFilter>")]
    #[serde(default)]
    pub sqlx_logging_level: Option<log::LevelFilter>,
    #[serde(default)]
    pub schema_search_path: Option<String>,
}

/// Corresponds to [log::LevelFilter], redefined in order to be able to
/// implement Deserialize for the type
///
/// See https://serde.rs/remote-derive.html as well as
/// https://docs.rs/serde_with/3/serde_with/guide/serde_as/index.html#using-serde_as-with-serdes-remote-derives
/// for more details.
#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(remote = "log::LevelFilter")]
pub enum LogLevelFilter {
    Off,
    Error,
    Warn,
    Info,
    Debug,
    Trace,
}

impl<'de> DeserializeAs<'de, log::LevelFilter> for LogLevelFilter {
    fn deserialize_as<D>(deserializer: D) -> Result<log::LevelFilter, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        LogLevelFilter::deserialize(deserializer)
    }
}

impl From<&DatabaseConfig> for ConnectOptions {
    fn from(value: &DatabaseConfig) -> ConnectOptions {
        let mut options = ConnectOptions::new(value.url.clone());
        if let Some(max_connections) = value.max_connections {
            options.max_connections(max_connections);
        }
        if let Some(min_connections) = value.min_connections {
            options.min_connections(min_connections);
        }
        if let Some(connect_timeout) = value.connect_timeout {
            options.connect_timeout(connect_timeout);
        }
        if let Some(idle_timeout) = value.idle_timeout {
            options.idle_timeout(idle_timeout);
        }
        if let Some(acquire_timeout) = value.acquire_timeout {
            options.acquire_timeout(acquire_timeout);
        }
        if let Some(max_lifetime) = value.max_lifetime {
            options.max_lifetime(max_lifetime);
        }
        if let Some(sqlx_logging_level) = value.sqlx_logging_level {
            options
                .sqlx_logging(sqlx_logging_level != log::LevelFilter::Off)
                .sqlx_logging_level(sqlx_logging_level);
        }
        if let Some(schema_search_path) = value.schema_search_path.clone() {
            options.set_schema_search_path(schema_search_path);
        }
        options
    }
}
