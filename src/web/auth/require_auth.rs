use crate::db::models::{UserAuthorization, UserBasics};
use crate::web::error::ApiError;
use crate::web::WebAppData;
use async_trait::async_trait;
use axum::extract::FromRequest;
use axum::http::Request;
use axum::Extension;
use chrono::Utc;
use http::StatusCode;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_AUTHORIZATION_HEADER: Regex = Regex::new("^Bearer ([0-9a-f]{128})$").unwrap();
}

pub struct PosssiblyExpiredUserAuthorization(pub UserAuthorization);

#[async_trait]
impl<S: Send + Sync, B: Send + 'static> FromRequest<S, B> for PosssiblyExpiredUserAuthorization {
    type Rejection = ApiError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let auth_header = req
            .headers()
            .get(http::header::AUTHORIZATION)
            .map(|header| header.to_str());
        let auth_header = match auth_header {
            Some(Ok(auth_header)) => auth_header,
            Some(Err(_)) => {
                return Err(ApiError::new_detailed(
                    StatusCode::BAD_REQUEST,
                    "header_value_not_utf8",
                    "Header value for Header `Authorization` was not valid UTF-8",
                ))
            }
            None => {
                return Err(ApiError::new_detailed(
                    StatusCode::BAD_REQUEST,
                    "missing_header",
                    "Missing header `Authorization`",
                ))
            }
        };

        let access_token = RE_AUTHORIZATION_HEADER
            .captures(auth_header)
            .ok_or_else(|| {
                ApiError::new_detailed(
                    StatusCode::BAD_REQUEST,
                    "malformed_header",
                    "Malformed `Authorization` header",
                )
            })?
            .get(1)
            .unwrap()
            .as_str()
            .to_owned();

        let app_data = Extension::<WebAppData>::from_request(req, state)
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

        let auth = PosssiblyExpiredUserAuthorization(UserAuthorization {
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
impl<S: Send + Sync, B: Send + 'static> FromRequest<S, B> for UserAuthorization {
    type Rejection = ApiError;

    async fn from_request(req: Request<B>, state: &S) -> Result<Self, Self::Rejection> {
        let auth = PosssiblyExpiredUserAuthorization::from_request(req, state).await?;

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
