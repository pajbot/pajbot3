use serde::Deserialize;
use std::net::SocketAddr;
#[cfg(unix)]
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum ListenAddr {
    #[serde(rename = "tcp")]
    Tcp { address: SocketAddr },
    #[cfg(unix)]
    #[serde(rename = "unix")]
    Unix { path: PathBuf },
}

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct WebConfig {
    pub listen_address: ListenAddr,
}

impl Default for WebConfig {
    fn default() -> Self {
        WebConfig {
            listen_address: ListenAddr::Tcp {
                address: "127.0.0.1:2791".parse().unwrap(),
            },
        }
    }
}
