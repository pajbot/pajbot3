mod get_token;
mod refresh;

pub use get_token::{get_token, GetTokenError};
pub use refresh::{refresh_token, RefreshTokenError};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Serialize)]
pub struct TwitchUserAccessToken {
    pub access_token: String,
    pub refresh_token: String,
    pub valid_until: DateTime<Utc>,
    pub scope: HashSet<String>,
}

#[derive(Deserialize)]
struct TwitchUserAccessTokenResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: i64,
    pub scope: Option<Vec<String>>,
}

impl From<TwitchUserAccessTokenResponse> for TwitchUserAccessToken {
    fn from(resp: TwitchUserAccessTokenResponse) -> TwitchUserAccessToken {
        TwitchUserAccessToken {
            access_token: resp.access_token,
            refresh_token: resp.refresh_token,
            valid_until: Utc::now() + Duration::seconds(resp.expires_in),
            scope: match resp.scope {
                None => HashSet::new(),
                Some(scope) => HashSet::from_iter(scope.into_iter()),
            },
        }
    }
}
