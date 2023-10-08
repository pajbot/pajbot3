use crate::api;
use crate::models::{user, user_authorization};
use crate::web::error::ApiError;
use crate::web::WebAppData;
use anyhow::Context;
use axum::extract::State;
use http::StatusCode;
use sea_orm::ModelTrait;

pub async fn revoke_token(
    State(app_data): State<WebAppData>,
    (auth, _): (user_authorization::PossiblyExpired, user::Model),
) -> Result<StatusCode, ApiError> {
    api::twitch::auth::revoke_token(&app_data.config.twitch_api, &auth.0.twitch_access_token)
        .await
        .context("revoke_token call twitch")?;

    auth.0
        .delete(app_data.db)
        .await
        .context("revoke_token delete from DB")?;

    Ok(StatusCode::NO_CONTENT)
}
