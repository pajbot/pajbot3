mod get_token;

pub use get_token::{get_token, GetTokenError};

use chrono::{DateTime, Utc};
use serde::Serialize;
use std::collections::HashSet;

#[derive(Serialize)]
pub struct UserAccessToken {
    pub access_token: String,
    pub refresh_token: String,
    pub valid_until: DateTime<Utc>,
    pub scope: HashSet<String>,
}
