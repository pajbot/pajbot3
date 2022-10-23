pub mod create;
pub mod refresh;
pub mod require_auth;
pub mod revoke;

use crate::api::twitch::user::UserDetails;
use chrono::{DateTime, Utc};
use serde::Serialize;

#[derive(Serialize)]
pub struct UserAuthorizationResponse {
    access_token: String,
    valid_until: DateTime<Utc>,
    user_details: UserDetails,
}
