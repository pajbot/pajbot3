use crate::api::twitch::auth::UserAccessToken;
use crate::api::twitch::ApiClientCredentials;
use chrono::{Duration, Utc};
use reqwest::StatusCode;
use serde::Deserialize;
use std::collections::HashSet;
use thiserror::Error;

#[derive(Deserialize)]
struct GetTokenResponse {
    access_token: String,
    refresh_token: String,
    expires_in: i64,
    scope: Option<Vec<String>>,
}

#[derive(Error, Debug)]
pub enum GetTokenError {
    #[error("Invalid authorization code")]
    InvalidAuthorizationCode(reqwest::Error),
    #[error("{0}")]
    Other(#[from] reqwest::Error),
}

pub async fn get_token(
    client_credentials: &ApiClientCredentials,
    code: &str,
) -> Result<UserAccessToken, GetTokenError> {
    let resp = crate::HTTP_CLIENT
        .post("https://id.twitch.tv/oauth2/token")
        .query(&[
            ("client_id", client_credentials.client_id.as_str()),
            ("client_secret", client_credentials.client_secret.as_str()),
            ("redirect_uri", client_credentials.redirect_uri.as_str()),
            ("code", code),
            ("grant_type", "authorization_code"),
        ])
        .send()
        .await?
        .error_for_status()
        .map_err(|e| {
            if e.status().unwrap() == StatusCode::BAD_REQUEST {
                GetTokenError::InvalidAuthorizationCode(e)
            } else {
                GetTokenError::Other(e)
            }
        })?
        .json::<GetTokenResponse>()
        .await?;

    Ok(UserAccessToken {
        access_token: resp.access_token,
        refresh_token: resp.refresh_token,
        valid_until: Utc::now() + Duration::seconds(resp.expires_in),
        scope: match resp.scope {
            None => HashSet::new(),
            Some(scope) => HashSet::from_iter(scope.into_iter()),
        },
    })
}
