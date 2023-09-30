pub mod create_login;
pub mod create_special;
pub mod refresh_login;
pub mod require_auth;
pub mod revoke_login;

use crate::api;
use crate::api::twitch::auth::{GetTokenError, TwitchUserAccessToken};
use crate::api::twitch::user::UserDetails;
use crate::db::models::UserBasics;
use crate::web::error::ApiError;
use crate::web::WebAppData;
use chrono::{DateTime, Utc};
use deadpool_postgres::Transaction;
use http::StatusCode;
use serde::Serialize;

#[derive(Serialize)]
pub struct UserAuthorizationResponse {
    access_token: String,
    valid_until: DateTime<Utc>,
    user_details: UserDetails,
}

pub async fn exchange_code(
    app_data: &WebAppData,
    code: &str,
) -> Result<(TwitchUserAccessToken, UserDetails), ApiError> {
    let twitch_user_access_token =
        match api::twitch::auth::get_token(&app_data.config.twitch_api, code).await {
            Ok(auth) => auth,
            Err(GetTokenError::InvalidAuthorizationCode(_)) => {
                return Err(ApiError::new_detailed(
                    StatusCode::BAD_REQUEST,
                    "invalid_authorization_code",
                    "Provided code could not be exchanged for a token, it is not valid",
                ))
            }
            Err(GetTokenError::Other(e)) => {
                return Err(e.into());
            }
        };

    let user_details = api::twitch::user::get_user_for_authorization(
        &app_data.config.twitch_api,
        &twitch_user_access_token.access_token,
    )
    .await?;
    Ok((twitch_user_access_token, user_details))
}

pub async fn upsert_user<'a>(
    user_basics: &'a UserBasics,
    tx: &'a Transaction<'a>,
) -> Result<(), tokio_postgres::Error> {
    tx.execute(
        r#"INSERT INTO "user"(id, login, display_name) VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET login = excluded.login, display_name = excluded.display_name"#,
        &[
            &user_basics.id,
            &user_basics.login,
            &user_basics.display_name,
        ],
    )
        .await.map(|_| ())
}
