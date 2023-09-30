use crate::db::models::{AuthorizationPurpose, UserBasics};
use crate::web::auth::exchange_code;
use crate::web::error::ApiError;
use crate::web::{auth, WebAppData};
use axum::extract::rejection::QueryRejection;
use axum::extract::Query;
use axum::{Extension, Json};
use lazy_static::lazy_static;
use maplit::hashset;
use serde::Deserialize;
use std::collections::HashSet;

lazy_static! {
    // TODO review these, make sure we're only requesting what we need
    static ref BOT_SCOPES: HashSet<&'static str> = hashset! {
        "moderator:manage:announcements",
        "moderator:manage:banned_users",
        "moderator:manage:chat_messages",
        "user:manage:whispers",
        "channel:moderate",
        "chat:edit",
        "chat:read",
        "whispers:read",
        "whispers:edit"
    };

    static ref BROADCASTER_SCOPES: HashSet<&'static str> = hashset! {
        "channel:read:subscriptions",
        "channel:manage:broadcast",
        "channel:read:vips",
        "moderation:read"
    };
}

#[derive(Deserialize)]
pub struct CreateSpecialAuthQueryOptions {
    code: String,
    purpose: String,
}

pub async fn create_special_twitch_auth(
    Extension(app_data): Extension<WebAppData>,
    query_options: Result<Query<CreateSpecialAuthQueryOptions>, QueryRejection>,
) -> Result<Json<UserBasics>, ApiError> {
    let query = query_options.map_err(|_| ApiError::bad_query_parameters())?;

    let purpose = AuthorizationPurpose::from_str(&query.purpose)
        .ok_or_else(|| ApiError::bad_query_parameters())?;

    let (twitch_user_access_token, user_details) = exchange_code(&app_data, &query.code).await?;

    let mut db_conn = app_data.db.get().await?;
    let tx = db_conn.transaction().await?;

    auth::upsert_user(&user_details.basics, &tx).await?;

    tx.execute(r#"
    INSERT INTO special_twitch_authorization(access_token, refresh_token, valid_until, user_id, purpose)
    VALUES ($1, $2, $3, $4, $5)"#, &[
        &twitch_user_access_token.access_token,
        &twitch_user_access_token.refresh_token,
        &twitch_user_access_token.valid_until,
        &user_details.basics.id,
        &purpose
    ]).await?;

    tx.commit().await?;

    Ok(Json(user_details.basics))
}
