use crate::api::twitch::ApiClientCredentials;
use crate::HTTP_CLIENT;
use http::StatusCode;
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RevokeTokenError {
    #[error("Invalid access token")]
    InvalidAccessToken(reqwest::Error),
    #[error("{0}")]
    Other(#[from] reqwest::Error),
}

#[derive(Serialize)]
struct RevokeFormData<'a> {
    client_id: &'a str,
    token: &'a str,
}

pub async fn revoke_token(
    client_credentials: &ApiClientCredentials,
    access_token: &str,
) -> Result<(), RevokeTokenError> {
    HTTP_CLIENT
        .post("https://id.twitch.tv/oauth2/revoke")
        .form(&RevokeFormData {
            client_id: &client_credentials.client_id,
            token: access_token,
        })
        .send()
        .await?
        .error_for_status()
        .map_err(|err| {
            if err.status() == Some(StatusCode::BAD_REQUEST) {
                RevokeTokenError::InvalidAccessToken(err)
            } else {
                RevokeTokenError::Other(err)
            }
        })?;
    Ok(())
}
