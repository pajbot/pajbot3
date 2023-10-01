use crate::api;
use crate::web::auth::require_auth::PossiblyExpiredUserAuthorization;
use crate::web::error::ApiError;
use crate::web::WebAppData;
use axum::Extension;
use http::StatusCode;

pub async fn revoke_token(
    Extension(app_data): Extension<WebAppData>,
    auth: PossiblyExpiredUserAuthorization,
) -> Result<StatusCode, ApiError> {
    api::twitch::auth::revoke_token(&app_data.config.twitch_api, &auth.0.twitch_access_token)
        .await?;

    app_data
        .db
        .get()
        .await?
        .execute(
            "DELETE FROM user_authorization WHERE access_token = $1",
            &[&auth.0.access_token],
        )
        .await?;

    Ok(StatusCode::NO_CONTENT)
}
