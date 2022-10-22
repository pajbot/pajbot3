use serde::Deserialize;
use std::time::Duration;
use tokio_postgres as postgres;

/// Database config
#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct DatabaseConfig {
    pub user: Option<String>,
    // psql seems to accept arbitrary bytes instead of just valid UTF-8 here
    // (the password in the tokio_postgres library is a Vec<u8>)
    // However since TOML does not support "raw" strings and you would have to type out an array
    // of bytes, using a String is my compromise.
    // Create a GitHub issue if you need non-UTF8 passwords.
    pub password: Option<String>,
    pub dbname: Option<String>,
    pub options: Option<String>,
    pub application_name: Option<String>,
    pub ssl_mode: PgSslMode,
    pub host: Vec<PgHost>,
    #[serde(with = "humantime_serde")]
    pub connect_timeout: Option<Duration>,
    pub keepalives: bool,
    #[serde(with = "humantime_serde")]
    pub keepalives_idle: Duration,
    pub target_session_attrs: PgTargetSessionAttrs,
    pub channel_binding: PgChannelBinding,
    #[serde(default)]
    pub pool: PoolConfig,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PgSslMode {
    Disable,
    Prefer,
    Require,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum PgHost {
    #[cfg(unix)]
    Unix {
        path: std::path::PathBuf,
        #[serde(default = "default_pg_port")]
        port: u16,
    },
    Tcp {
        hostname: String,
        #[serde(default = "default_pg_port")]
        port: u16,
    },
}

fn default_pg_port() -> u16 {
    5432
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PgTargetSessionAttrs {
    Any,
    ReadWrite,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PgChannelBinding {
    Disable,
    Prefer,
    Require,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        DatabaseConfig::from(postgres::Config::default())
    }
}

impl From<postgres::Config> for DatabaseConfig {
    fn from(config: postgres::Config) -> DatabaseConfig {
        let ports: Box<dyn Iterator<Item = u16>> = if config.get_ports().len() == 1 {
            Box::new(itertools::repeat_n(
                config.get_ports()[0],
                config.get_hosts().len(),
            ))
        } else {
            Box::new(itertools::cloned(config.get_ports().iter()))
        };

        let mut hosts = vec![];
        for (host, port) in config.get_hosts().iter().zip(ports) {
            let new_host = match host {
                postgres::config::Host::Tcp(hostname) => PgHost::Tcp {
                    hostname: hostname.to_owned(),
                    port,
                },
                #[cfg(unix)]
                postgres::config::Host::Unix(path) => PgHost::Unix {
                    path: path.clone(),
                    port,
                },
            };
            hosts.push(new_host);
        }

        DatabaseConfig {
            user: config.get_user().map(String::from),
            password: config
                .get_password()
                .map(|p| String::from_utf8_lossy(p).into_owned()),
            dbname: config.get_dbname().map(String::from),
            options: config.get_options().map(String::from),
            application_name: config.get_application_name().map(String::from),
            ssl_mode: match config.get_ssl_mode() {
                postgres::config::SslMode::Disable => PgSslMode::Disable,
                postgres::config::SslMode::Prefer => PgSslMode::Prefer,
                postgres::config::SslMode::Require => PgSslMode::Require,
                _ => panic!("unhandled variant"),
            },
            host: hosts,
            connect_timeout: config.get_connect_timeout().cloned(),
            keepalives: config.get_keepalives(),
            keepalives_idle: config.get_keepalives_idle(),
            target_session_attrs: match config.get_target_session_attrs() {
                postgres::config::TargetSessionAttrs::Any => PgTargetSessionAttrs::Any,
                postgres::config::TargetSessionAttrs::ReadWrite => PgTargetSessionAttrs::ReadWrite,
                _ => panic!("unhandled variant"),
            },
            channel_binding: match config.get_channel_binding() {
                postgres::config::ChannelBinding::Disable => PgChannelBinding::Disable,
                postgres::config::ChannelBinding::Prefer => PgChannelBinding::Prefer,
                postgres::config::ChannelBinding::Require => PgChannelBinding::Require,
                _ => panic!("unhandled variant"),
            },
            pool: PoolConfig::default(),
        }
    }
}

impl From<DatabaseConfig> for postgres::Config {
    fn from(config: DatabaseConfig) -> Self {
        let mut new_cfg = postgres::Config::new();
        if let Some(ref user) = config.user {
            new_cfg.user(user);
        }
        if let Some(ref password) = config.password {
            new_cfg.password(password);
        }
        if let Some(ref dbname) = config.dbname {
            new_cfg.dbname(dbname);
        }
        if let Some(ref options) = config.options {
            new_cfg.dbname(options);
        }
        if let Some(ref application_name) = config.application_name {
            new_cfg.application_name(application_name);
        } else {
            new_cfg.application_name("recent-messages2");
        }
        new_cfg.ssl_mode(match config.ssl_mode {
            PgSslMode::Disable => postgres::config::SslMode::Disable,
            PgSslMode::Prefer => postgres::config::SslMode::Prefer,
            PgSslMode::Require => postgres::config::SslMode::Require,
        });
        for host in config.host {
            match host {
                PgHost::Tcp { ref hostname, port } => {
                    new_cfg.host(hostname);
                    new_cfg.port(port);
                }
                #[cfg(unix)]
                PgHost::Unix { ref path, port } => {
                    new_cfg.host_path(path);
                    new_cfg.port(port);
                }
            }
        }

        if let Some(ref connect_timeout) = config.connect_timeout {
            new_cfg.connect_timeout(*connect_timeout);
        }
        new_cfg.keepalives(config.keepalives);
        new_cfg.keepalives_idle(config.keepalives_idle);
        new_cfg.target_session_attrs(match config.target_session_attrs {
            PgTargetSessionAttrs::Any => postgres::config::TargetSessionAttrs::Any,
            PgTargetSessionAttrs::ReadWrite => postgres::config::TargetSessionAttrs::ReadWrite,
        });
        new_cfg.channel_binding(match config.channel_binding {
            PgChannelBinding::Disable => postgres::config::ChannelBinding::Disable,
            PgChannelBinding::Prefer => postgres::config::ChannelBinding::Prefer,
            PgChannelBinding::Require => postgres::config::ChannelBinding::Require,
        });

        new_cfg
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct PoolConfig {
    pub max_size: usize,
    #[serde(with = "humantime_serde")]
    pub create_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub wait_timeout: Duration,
    #[serde(with = "humantime_serde")]
    pub recycle_timeout: Duration,
}

impl Default for PoolConfig {
    fn default() -> Self {
        PoolConfig {
            max_size: num_cpus::get() * 4,
            create_timeout: Duration::from_secs(5),
            wait_timeout: Duration::from_secs(5),
            recycle_timeout: Duration::from_secs(5),
        }
    }
}

impl From<PoolConfig> for deadpool_postgres::Timeouts {
    fn from(cfg: PoolConfig) -> Self {
        deadpool_postgres::Timeouts {
            create: Some(cfg.create_timeout),
            wait: Some(cfg.wait_timeout),
            recycle: Some(cfg.recycle_timeout),
        }
    }
}
