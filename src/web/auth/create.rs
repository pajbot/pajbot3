use crate::api;
use crate::api::twitch::auth::GetTokenError;
use crate::web::auth::UserAuthorizationResponse;
use crate::web::error::ApiError;
use crate::web::WebAppData;
use axum::extract::rejection::QueryRejection;
use axum::extract::Query;
use axum::{Extension, Json};
use chrono::{Duration, Utc};
use hyper::StatusCode;
use rand::distributions::Standard;
use rand::Rng;
use serde::Deserialize;
use std::fmt::Write;

#[derive(Deserialize)]
pub struct CreateAuthTokenQueryOptions {
    code: String,
}

// POST /api/v1/auth/create?code=abcdef123456
pub async fn create_token(
    Extension(app_data): Extension<WebAppData>,
    query_options: Result<Query<CreateAuthTokenQueryOptions>, QueryRejection>,
) -> Result<Json<UserAuthorizationResponse>, ApiError> {
    let code = &query_options
        .map_err(|_| ApiError::bad_query_parameters())?
        .code;

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

    // 512 bit random hex string
    // thread_rng() is cryptographically safe
    let access_token = rand::thread_rng().sample_iter(Standard).take(512 / 8).fold(
        String::with_capacity(512 / 4),
        |mut s, x: u8| {
            // format as hex, padded with a leading 0 if needed (e.g. 0x0 -> "00", 0xFF -> "ff")
            write!(&mut s, "{:02x}", x).unwrap();
            s
        },
    );

    let mut db_conn = app_data.db.get().await?;
    let tx = db_conn.transaction().await?;

    tx.execute(
        r#"INSERT INTO "user"(id, login, display_name) VALUES ($1, $2, $3)
        ON CONFLICT (id) DO UPDATE SET login = excluded.login, display_name = excluded.display_name"#,
        &[
            &user_details.id,
            &user_details.login,
            &user_details.display_name,
        ],
    )
    .await?;

    // tokens are supposed to be valid for a maximum of one hour.
    // See https://dev.twitch.tv/docs/authentication/validate-tokens#who-must-validate-tokens
    // We simply force a refresh after an hour
    let mut valid_until = Utc::now() + Duration::hours(1);
    if twitch_user_access_token.valid_until < valid_until {
        valid_until = twitch_user_access_token.valid_until;
    }

    tx.execute(r#"
    INSERT INTO user_authorization(access_token, twitch_access_token, twitch_refresh_token, valid_until, user_id)
    VALUES ($1, $2, $3, $4, $5)"#, &[
        &access_token,
        &twitch_user_access_token.access_token,
        &twitch_user_access_token.refresh_token,
        &valid_until,
        &user_details.id
    ]).await?;

    tx.commit().await?;

    Ok(Json(UserAuthorizationResponse {
        access_token,
        valid_until,
        user_details,
    }))
}
