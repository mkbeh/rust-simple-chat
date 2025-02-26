use axum::Json;
use chrono::{Duration, Utc};

use crate::{
    core_utils::{errors::ServerError, jwt, jwt::Claims},
    entities,
};

/// Login
///
/// Retrieve access token.
#[utoipa::path(
    post,
    path = "/login",
    tag = super::DOCS_AUTH_TAG,
    responses(
            (status = 200, description = "List all todos successfully", body = entities::auth::LoginResponse)
    )
)]
pub async fn login_handler() -> Result<Json<entities::auth::LoginResponse>, ServerError> {
    const USER_ID: i32 = 123;
    const TOKEN_LIFETIME_SECS: i64 = 60;

    let claims = Claims {
        sub: USER_ID.to_string(),
        exp: (Utc::now() + Duration::seconds(TOKEN_LIFETIME_SECS)).timestamp() as usize,
    };
    let token = jwt::encode_token(&claims)?;

    Ok(Json::from(entities::auth::LoginResponse { token }))
}
