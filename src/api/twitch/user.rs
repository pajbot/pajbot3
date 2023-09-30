use crate::api::twitch::ApiClientCredentials;
use crate::db::models::UserBasics;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct HelixGetUserResponse {
    // we expect a list of size 1
    pub data: (UserDetails,),
}

#[derive(Serialize, Deserialize)]
pub struct UserDetails {
    #[serde(flatten)]
    pub basics: UserBasics,
    #[serde(rename = "type")]
    pub user_type: String,
    pub broadcaster_type: String,
    pub description: String,
    pub profile_image_url: String,
    pub offline_image_url: String,
    pub view_count: u64,
    pub created_at: DateTime<Utc>,
}

pub async fn get_user_for_authorization(
    client_credentials: &ApiClientCredentials,
    access_token: &str,
) -> Result<UserDetails, reqwest::Error> {
    Ok(crate::HTTP_CLIENT
        .get("https://api.twitch.tv/helix/users")
        .header("Client-Id", &client_credentials.client_id)
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?
        .error_for_status()?
        .json::<HelixGetUserResponse>()
        .await?
        .data
        .0)
}
