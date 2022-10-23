///! Data models as they are stored in the database.
use chrono::{DateTime, Utc};

#[derive(Debug)]
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
