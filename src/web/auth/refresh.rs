use axum::Extension;
use crate::web::auth::UserAuthorizationResponse;
use crate::web::WebAppData;

// POST /api/v1/auth/extend
pub async fn refresh_token(
    Extension(app_data): Extension<WebAppData>,
    query_options: Result<Query<CreateAuthTokenQueryOptions>, QueryRejection>,
) -> Result<Json<UserAuthorizationResponse>, ApiError> {

}
