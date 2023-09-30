use crate::api;
use crate::api::twitch::auth::RefreshTokenError;
use crate::web::auth::require_auth::PosssiblyExpiredUserAuthorization;
use crate::web::auth::{upsert_user, UserAuthorizationResponse};
use crate::web::error::ApiError;
use crate::web::WebAppData;
use axum::{Extension, Json};
use chrono::{Duration, Utc};
use http::StatusCode;

// POST /api/v1/auth/extend
pub async fn refresh_token(
    Extension(app_data): Extension<WebAppData>,
    auth: PosssiblyExpiredUserAuthorization,
) -> Result<Json<UserAuthorizationResponse>, ApiError> {
    let new_twitch_auth =
        api::twitch::auth::refresh_token(&app_data.config.twitch_api, &auth.0.twitch_refresh_token)
            .await
            .map_err(|err| {
                match err {
        RefreshTokenError::InvalidRefreshToken(_) => ApiError::new_detailed(
            StatusCode::UNAUTHORIZED,
            "invalid_auth",
            "The Twitch authorization associated with this authorization has been invalidated",
        ),
        RefreshTokenError::Other(e) => ApiError::from(e),
                }
            })?;

    // also refresh user details (their name, profile picture, etc.)
    let user_details = api::twitch::user::get_user_for_authorization(
        &app_data.config.twitch_api,
        &new_twitch_auth.access_token,
    )
    .await?;

    let mut db_conn = app_data.db.get().await?;
    let tx = db_conn.transaction().await?;

    upsert_user(&user_details.basics, &tx).await?;

    // tokens are supposed to be valid for a maximum of one hour.
    // See https://dev.twitch.tv/docs/authentication/validate-tokens#who-must-validate-tokens
    // We simply force a refresh after an hour
    let mut valid_until = Utc::now() + Duration::hours(1);
    if new_twitch_auth.valid_until < valid_until {
        valid_until = new_twitch_auth.valid_until;
    }

    tx.execute(
        r#"
    UPDATE user_authorization
    SET twitch_access_token = $1,
    twitch_refresh_token = $2,
    valid_until = $3,
    user_id = $4
    WHERE access_token = $5"#,
        &[
            &new_twitch_auth.access_token,
            &new_twitch_auth.refresh_token,
            &valid_until,
            &user_details.basics.id,
            &auth.0.access_token,
        ],
    )
    .await?;

    tx.commit().await?;

    Ok(Json(UserAuthorizationResponse {
        access_token: auth.0.access_token,
        valid_until,
        user_details,
    }))
}
