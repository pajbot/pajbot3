pub mod create;
//pub mod refresh;
//pub mod require_auth;

use chrono::{DateTime, Utc};
use serde::Serialize;
use crate::api::twitch::user::UserDetails;

#[derive(Serialize)]
pub struct UserAuthorizationResponse {
    access_token: String,
    valid_until: DateTime<Utc>,
    user_details: UserDetails,
}
