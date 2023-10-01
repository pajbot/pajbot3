use crate::db::models::{UserAuthorization, UserBasics};
use crate::web::error::ApiError;
use crate::web::WebAppData;
use anyhow::anyhow;
use async_trait::async_trait;
use axum::extract::rejection::TypedHeaderRejectionReason;
use axum::extract::{FromRequestParts, State, TypedHeader};
use axum::headers::{authorization::Bearer, Authorization};
use chrono::Utc;
use http::request::Parts;
use http::StatusCode;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_AUTHORIZATION_HEADER: Regex = Regex::new("^Bearer ([0-9a-f]{128})$").unwrap();
}

pub struct PossiblyExpiredUserAuthorization(pub UserAuthorization);

#[async_trait]
impl FromRequestParts<WebAppData> for PossiblyExpiredUserAuthorization {
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
        // let auth_header = match auth_header {
        //     Some(Ok(auth_header)) => auth_header,
        //     Some(Err(_)) => {
        //         return Err(ApiError::new_detailed(
        //             StatusCode::BAD_REQUEST,
        //             "header_value_not_utf8",
        //             "Header value for Header `Authorization` was not valid UTF-8",
        //         ))
        //     }
        //     None => {
        //         return Err(ApiError::new_detailed(
        //             StatusCode::BAD_REQUEST,
        //             "missing_header",
        //             "Missing header `Authorization`",
        //         ))
        //     }
        // };

        // let access_token = RE_AUTHORIZATION_HEADER
        //     .captures(auth_header.token)
        //     .ok_or_else(|| {
        //         ApiError::new_detailed(
        //             StatusCode::BAD_REQUEST,
        //             "malformed_header",
        //             "Malformed `Authorization` header",
        //         )
        //     })?
        //     .get(1)
        //     .unwrap()
        //     .as_str()
        //     .to_owned();
        let access_token = auth_header.token().to_owned();

        let State(app_data) = State::<WebAppData>::from_request_parts(parts, state)
            .await
            .unwrap();
        let row = app_data
            .db
            .get()
            .await?
            .query_opt(
                r#"SELECT user_authorization.twitch_access_token,
       user_authorization.twitch_refresh_token,
       user_authorization.valid_until,
       user_authorization.user_id,
       "user".login,
       "user".display_name
FROM user_authorization
JOIN "user" ON user_authorization.user_id = "user".id
WHERE access_token = $1"#,
                &[&access_token],
            )
            .await?
            .ok_or_else(|| {
                ApiError::new_detailed(
                    StatusCode::UNAUTHORIZED,
                    "access_token_invalid",
                    "Unauthorized (access token invalid)",
                )
            })?;

        let auth = PossiblyExpiredUserAuthorization(UserAuthorization {
            access_token,
            twitch_access_token: row.get(0),
            twitch_refresh_token: row.get(1),
            valid_until: row.get(2),
            user: UserBasics {
                id: row.get(3),
                login: row.get(4),
                display_name: row.get(5),
            },
        });

        Ok(auth)
    }
}

#[async_trait]
impl FromRequestParts<WebAppData> for UserAuthorization {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &WebAppData,
    ) -> Result<Self, Self::Rejection> {
        let auth = PossiblyExpiredUserAuthorization::from_request_parts(parts, state).await?;

        if Utc::now() > auth.0.valid_until {
            return Err(ApiError::new_detailed(
                StatusCode::UNAUTHORIZED,
                "access_token_expired",
                "Unauthorized (access token has expired)",
            ));
        }

        Ok(auth.0)
    }
}
