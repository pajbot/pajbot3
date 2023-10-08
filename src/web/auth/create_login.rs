use crate::models::{user, user_authorization};
use crate::web::auth::UserAuthorizationResponse;
use crate::web::error::ApiError;
use crate::web::{auth, WebAppData};
use anyhow::Context;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use axum::Json;
use chrono::{Duration, Utc};
use rand::distributions::Standard;
use rand::Rng;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, TransactionTrait};
use serde::Deserialize;
use std::fmt::Write;

#[derive(Deserialize)]
pub struct CreateAuthTokenQueryOptions {
    code: String,
}

// POST /api/v1/auth/create?code=abcdef123456
pub async fn create_token(
    State(app_data): State<WebAppData>,
    query_options: Result<Query<CreateAuthTokenQueryOptions>, QueryRejection>,
) -> Result<Json<UserAuthorizationResponse>, ApiError> {
    let code = &query_options
        .map_err(|_| ApiError::bad_query_parameters())?
        .code;

    let (twitch_user_access_token, user_details) = auth::exchange_code(&app_data, code).await?;

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

    let tx = app_data.db.begin().await?;
    user::upsert_user(user_details.basics.clone(), &tx)
        .await
        .context("create_token upsert user")?;

    // tokens are supposed to be valid for a maximum of one hour.
    // See https://dev.twitch.tv/docs/authentication/validate-tokens#who-must-validate-tokens
    // We simply force a refresh after an hour
    let mut valid_until = Utc::now() + Duration::hours(1);
    if twitch_user_access_token.valid_until < valid_until {
        valid_until = twitch_user_access_token.valid_until;
    }

    let user_authorization = user_authorization::ActiveModel {
        access_token: Set(access_token.clone()),
        twitch_access_token: Set(twitch_user_access_token.access_token),
        twitch_refresh_token: Set(twitch_user_access_token.refresh_token),
        valid_until: Set(valid_until),
        user_id: Set(user_details.basics.id.clone()),
    };
    user_authorization
        .insert(&tx)
        .await
        .context("create_token insert user_authorization")?;
    tx.commit().await?;

    Ok(Json(UserAuthorizationResponse {
        access_token,
        valid_until,
        user_details,
    }))
}
