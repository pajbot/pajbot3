use crate::api::twitch::auth::{TwitchUserAccessToken, TwitchUserAccessTokenResponse};
use crate::api::twitch::ApiClientCredentials;
use crate::HTTP_CLIENT;
use http::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RefreshTokenError {
    #[error("Invalid refresh token")]
    InvalidRefreshToken(reqwest::Error),
    #[error("{0}")]
    Other(#[from] reqwest::Error),
}

#[derive(Serialize)]
struct RefreshFormData<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    grant_type: &'static str,
    refresh_token: &'a str,
}

pub async fn refresh_token(
    client_credentials: &ApiClientCredentials,
    refresh_token: &str,
) -> Result<TwitchUserAccessToken, RefreshTokenError> {
    let resp = HTTP_CLIENT
        .post("https://id.twitch.tv/oauth2/token")
        .form(&RefreshFormData {
            client_id: &client_credentials.client_id,
            client_secret: &client_credentials.client_secret,
            grant_type: "refresh_token",
            refresh_token,
        })
        .send()
        .await?
        .error_for_status()
        .map_err(|err| {
            if err.status() == Some(StatusCode::BAD_REQUEST) {
                RefreshTokenError::InvalidRefreshToken(err)
            } else {
                RefreshTokenError::Other(err)
            }
        })?
        .json::<TwitchUserAccessTokenResponse>()
        .await?;

    Ok(TwitchUserAccessToken::from(resp))
}
