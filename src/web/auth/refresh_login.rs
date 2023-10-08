use crate::api;
use crate::api::twitch::auth::RefreshTokenError;
use crate::models::{user, user_authorization};
use crate::web::auth::UserAuthorizationResponse;
use crate::web::error::ApiError;
use crate::web::WebAppData;
use axum::extract::State;
use axum::Json;
use chrono::{Duration, Utc};
use http::StatusCode;
use sea_orm::ActiveValue::Set;
use sea_orm::{ActiveModelTrait, TransactionTrait};

// POST /api/v1/auth/extend
pub async fn refresh_token(
    State(app_data): State<WebAppData>,
    (auth, _): (user_authorization::PossiblyExpired, user::Model),
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

    let tx = app_data.db.begin().await?;

    user::upsert_user(user_details.basics.clone(), &tx).await?;

    // tokens are supposed to be valid for a maximum of one hour.
    // See https://dev.twitch.tv/docs/authentication/validate-tokens#who-must-validate-tokens
    // We simply force a refresh after an hour
    let mut valid_until = Utc::now() + Duration::hours(1);
    if new_twitch_auth.valid_until < valid_until {
        valid_until = new_twitch_auth.valid_until;
    }

    let fresh_auth = user_authorization::ActiveModel {
        access_token: Set(auth.0.access_token.clone()),
        twitch_access_token: Set(new_twitch_auth.access_token),
        twitch_refresh_token: Set(new_twitch_auth.refresh_token),
        valid_until: Set(valid_until),
        user_id: Set(user_details.basics.id.clone()),
    };
    fresh_auth.update(&tx).await?;
    tx.commit().await?;

    Ok(Json(UserAuthorizationResponse {
        access_token: auth.0.access_token,
        valid_until,
        user_details,
    }))
}
