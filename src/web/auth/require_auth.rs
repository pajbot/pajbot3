use crate::web::error::ApiError;
use crate::web::WebAppData;
use axum::middleware::Next;
use axum::response::IntoResponse;
use http::{Request, StatusCode};
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_AUTHORIZATION_HEADER: Regex = Regex::new("^Bearer ([0-9a-f]{128})$").unwrap();
}

pub async fn require_authorization<B>(
    mut req: Request<B>,
    next: Next<B>,
    app_data: WebAppData,
) -> impl IntoResponse {
    let auth_header = req
        .headers()
        .get(http::header::AUTHORIZATION)
        .map(|header| header.to_str());
    let auth_header = match auth_header {
        Some(Ok(auth_header)) => auth_header,
        Some(Err(_)) => return Err(ApiError::new_detailed(StatusCode::BAD_REQUEST, "header_value_not_utf8", "Header value for Header `Authorization` was not valid UTF-8")),
        None => return Err(ApiError::new_detailed(StatusCode::BAD_REQUEST, "missing_header", "Missing header `Authorization`")),
    };

    let access_token = RE_AUTHORIZATION_HEADER
        .captures(&auth_header)
        .ok_or_else(|| ApiError::new_detailed(StatusCode::BAD_REQUEST, "malformed_header", "Malformed `Authorization` header"))?
        .get(1)
        .unwrap()
        .as_str();

    // data storage query ensures token is not totally expired
    let mut authorization = app_data
        .data_storage
        .get_user_authorization(access_token)
        .await
        .map_err(ApiError::map_internal("require_authorization query for authorization"))?
        .ok_or_else(|| ApiError::new_detailed(StatusCode::UNAUTHORIZED, "unauthorized", "Unauthorized (access token expired or invalid)"))?;

    // and then this ensures that the user has not revoked the connection from the Twitch side
    let pre_validation_auth = authorization.clone();
    authorization
        .validate_still_valid(
            &app_data.config.web.twitch_api_credentials,
            app_data.config.web.recheck_twitch_auth_after,
        )
        .await?;

    if pre_validation_auth != authorization {
        app_data
            .data_storage
            .update_user_authorization(&authorization)
            .await
            .map_err(ApiError::UpdateUserAuthorization)?;
    }

    req.extensions_mut().insert(authorization);

    Ok(next.run(req).await)
}
