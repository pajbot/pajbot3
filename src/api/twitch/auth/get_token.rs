use crate::api::twitch::auth::{TwitchUserAccessToken, TwitchUserAccessTokenResponse};
use crate::api::twitch::ApiClientCredentials;
use reqwest::StatusCode;
use thiserror::Error;

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
) -> Result<TwitchUserAccessToken, GetTokenError> {
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
        .json::<TwitchUserAccessTokenResponse>()
        .await?;

    Ok(TwitchUserAccessToken::from(resp))
}
