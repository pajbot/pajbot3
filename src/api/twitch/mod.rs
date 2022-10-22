pub mod auth;
pub mod user;

use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiClientCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
}
