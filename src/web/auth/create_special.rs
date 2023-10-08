use crate::models::{special_twitch_authorization, user};
use crate::web::auth::exchange_code;
use crate::web::error::ApiError;
use crate::web::WebAppData;
use axum::extract::rejection::QueryRejection;
use axum::extract::{Query, State};
use http::StatusCode;
use lazy_static::lazy_static;
use maplit::hashset;
use sea_orm::ActiveValue::Set;
use sea_orm::TransactionTrait;
use serde::Deserialize;
use std::collections::HashSet;

lazy_static! {
    // TODO review these, make sure we're only requesting what we need
    static ref BOT_SCOPE: HashSet<&'static str> = hashset! {
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

    static ref BROADCASTER_SCOPE: HashSet<&'static str> = hashset! {
        "channel:read:subscriptions",
        "channel:manage:broadcast",
        "channel:read:vips",
        "moderation:read"
    };
}

// Increase these constants whenever a scope gets added to the sets above. No version
// upgrade necessary when something is removed from the set, though.
const BOT_SCOPE_VERSION: i16 = 1;
const BROADCASTER_SCOPE_VERSION: i16 = 1;

#[derive(Deserialize)]
pub struct CreateSpecialAuthQueryOptions {
    code: String,
}

pub async fn create_special_twitch_auth(
    State(app_data): State<WebAppData>,
    query_options: Result<Query<CreateSpecialAuthQueryOptions>, QueryRejection>,
) -> Result<(), ApiError> {
    let query = query_options.map_err(|_| ApiError::bad_query_parameters())?;

    let (twitch_user_access_token, user_details) = exchange_code(&app_data, &query.code).await?;
    let is_bot = BOT_SCOPE
        .iter()
        .all(|s| twitch_user_access_token.scope.contains(*s));
    let is_broadcaster = BROADCASTER_SCOPE
        .iter()
        .all(|s| twitch_user_access_token.scope.contains(*s));
    if !(is_bot || is_broadcaster) {
        return Err(ApiError::new_detailed(StatusCode::BAD_REQUEST, "auth_insufficient_scope", "Authorization does not provide sufficient scope to be either a valid Bot or Broadcaster authorization"));
    }

    let tx = app_data.db.begin().await?;
    user::upsert_user(user_details.basics.clone(), &tx).await?;

    let special_twitch_authorization = special_twitch_authorization::ActiveModel {
        user_id: Set(user_details.basics.id),
        bot_scope_version: Set(if is_bot {
            Some(BOT_SCOPE_VERSION)
        } else {
            None
        }),
        broadcaster_scope_version: Set(if is_broadcaster {
            Some(BROADCASTER_SCOPE_VERSION)
        } else {
            None
        }),
        twitch_access_token: Set(twitch_user_access_token.access_token),
        twitch_refresh_token: Set(twitch_user_access_token.refresh_token),
        valid_until: Set(twitch_user_access_token.valid_until),
    };
    special_twitch_authorization::upsert(special_twitch_authorization, &tx).await?;
    tx.commit().await?;

    Ok(())
}
