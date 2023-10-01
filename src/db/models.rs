//! Data models as they are stored in the database.
use chrono::{DateTime, Utc};
use postgres_types::{FromSql, ToSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct UserBasics {
    pub id: String,
    pub login: String,
    pub display_name: String,
}

#[derive(Debug)]
pub struct UserAuthorization {
    pub access_token: String,
    pub twitch_access_token: String,
    pub twitch_refresh_token: String,
    pub valid_until: DateTime<Utc>,
    pub user: UserBasics,
}

#[derive(Debug, ToSql, FromSql)]
#[postgres(name = "authorization_purpose")]
pub enum AuthorizationPurpose {
    #[postgres(name = "bot")]
    Bot,
    #[postgres(name = "broadcaster")]
    Broadcaster,
}

impl AuthorizationPurpose {
    pub fn from_str(key: &str) -> Option<AuthorizationPurpose> {
        match key {
            "bot" => Some(AuthorizationPurpose::Bot),
            "broadcaster" => Some(AuthorizationPurpose::Broadcaster),
            _ => None,
        }
    }
}
