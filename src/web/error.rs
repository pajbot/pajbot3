use axum::response::{IntoResponse, Response};
use axum::Json;
use reqwest::StatusCode;
use serde::Serialize;
use std::borrow::Cow;
use std::panic::Location;

/// Utility for the purpose of returning errors from the API in a consistent fashion.
#[derive(Debug, Serialize)]
pub struct ApiError {
    #[serde(skip)]
    pub status_code: StatusCode,
    pub details: Option<ApiErrorDetails>,
}

#[derive(Debug, Serialize)]
pub struct ApiErrorDetails {
    pub error_code: &'static str,
    pub user_message: Cow<'static, str>,
}

impl ApiError {
    pub fn new_detailed(
        status_code: StatusCode,
        error_code: &'static str,
        user_message: impl Into<Cow<'static, str>>,
    ) -> ApiError {
        ApiError {
            status_code,
            details: Some(ApiErrorDetails {
                error_code,
                user_message: user_message.into(),
            }),
        }
    }

    pub fn new_basic(status_code: StatusCode) -> ApiError {
        ApiError {
            status_code,
            details: None,
        }
    }

    pub fn bad_query_parameters() -> ApiError {
        ApiError::new_detailed(
            StatusCode::BAD_REQUEST,
            "bad_query_parameters",
            "Invalid or missing query parameters",
        )
    }

    pub fn method_not_allowed() -> ApiError {
        ApiError::new_basic(StatusCode::METHOD_NOT_ALLOWED)
    }
}

impl<E> From<E> for ApiError
where
    E: Into<anyhow::Error>,
{
    #[track_caller]
    fn from(e: E) -> ApiError {
        let caller = Location::caller();
        // The error is turned into a anyhow::Error to get the opportunity to print a backtrace
        // with RUST_BACKTRACE=1 (see also documentation on anyhow for more details)
        let anyhow_error: anyhow::Error = e.into();
        tracing::error!("({}) {:?}", caller, anyhow_error);
        ApiError::new_basic(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status_code, Json(self)).into_response()
    }
}
