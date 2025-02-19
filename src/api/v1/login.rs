use axum::Json;
use chrono::{Duration, Utc};

use crate::core_utils::errors::ServerError;
use crate::core_utils::jwt;
use crate::core_utils::jwt::Claims;
use crate::entities;

pub async fn login() -> Result<Json<entities::auth::LoginResponse>, ServerError> {
    const USER_ID: i32 = 123;
    const TOKEN_LIFETIME_SECS: i64 = 60;

    let claims = Claims {
        sub: USER_ID.to_string(),
        exp: (Utc::now() + Duration::seconds(TOKEN_LIFETIME_SECS)).timestamp() as usize,
    };
    let token = jwt::encode_token(&claims)?;

    Ok(Json::from(entities::auth::LoginResponse { token }))
}
