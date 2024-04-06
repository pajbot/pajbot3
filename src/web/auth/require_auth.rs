use crate::models::{user, user_authorization};
use crate::web::error::ApiError;
use crate::web::WebAppData;
use anyhow::{anyhow, Context};
use async_trait::async_trait;
use axum::extract::{FromRequestParts, State};
use axum_extra::headers::authorization::Bearer;
use axum_extra::headers::Authorization;
use axum_extra::typed_header::TypedHeaderRejectionReason;
use axum_extra::TypedHeader;
use chrono::Utc;
use http::request::Parts;
use http::StatusCode;
use sea_orm::EntityTrait;

#[async_trait]
impl FromRequestParts<WebAppData> for (user_authorization::PossiblyExpired, user::Model) {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &WebAppData,
    ) -> Result<Self, Self::Rejection> {
        let auth_header = TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
            .await
            .map_err(|err| match err.reason() {
                TypedHeaderRejectionReason::Missing => ApiError::new_detailed(
                    StatusCode::BAD_REQUEST,
                    "missing_header",
                    "Missing header `Authorization`",
                ),
                TypedHeaderRejectionReason::Error(_) => ApiError::new_detailed(
                    StatusCode::BAD_REQUEST,
                    "malformed_header",
                    "Malformed `Authorization` header",
                ),
                _ => anyhow!("Unknown TypedHeaderRejectionReason").into(),
            })?;

        let access_token = auth_header.token().to_owned();

        let State(app_data) = State::<WebAppData>::from_request_parts(parts, state)
            .await
            .unwrap();

        let (auth, user) = user_authorization::Entity::find_by_id(&access_token)
            .find_also_related(user::Entity)
            .one(app_data.db)
            .await
            .context("require_auth find authorization")?
            .ok_or_else(|| {
                ApiError::new_detailed(
                    StatusCode::UNAUTHORIZED,
                    "access_token_invalid",
                    "Unauthorized (access token invalid)",
                )
            })?;

        let auth = user_authorization::PossiblyExpired(auth);

        Ok((
            auth,
            user.expect("DB failed to enforce foreign key constraint"),
        ))
    }
}

#[async_trait]
impl FromRequestParts<WebAppData> for (user_authorization::Model, user::Model) {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &WebAppData,
    ) -> Result<Self, Self::Rejection> {
        let (auth, user) = <(user_authorization::PossiblyExpired, user::Model) as FromRequestParts<WebAppData>>::from_request_parts(parts, state).await?;

        if Utc::now() > auth.0.valid_until {
            return Err(ApiError::new_detailed(
                StatusCode::UNAUTHORIZED,
                "access_token_expired",
                "Unauthorized (access token has expired)",
            ));
        }

        Ok((auth.0, user))
    }
}
