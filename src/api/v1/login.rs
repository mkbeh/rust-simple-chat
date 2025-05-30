use std::sync::Arc;

use anyhow::anyhow;
use axum::{Extension, Json};
use caslex::{errors::DefaultError, middlewares::auth::Claims};
use caslex_extra::security::jwt;

use crate::{api::State, entities};

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
pub async fn login_handler(
    Extension(_state): Extension<Arc<State>>,
) -> Result<Json<entities::auth::LoginResponse>, DefaultError> {
    const USER_ID: i32 = 123;
    const TOKEN_LIFETIME_SECS: u64 = 300;

    let claims = Claims {
        sub: USER_ID.to_string(),
        exp: jwt::expiry(TOKEN_LIFETIME_SECS),
    };

    let token = match jwt::encode_token(&claims) {
        Ok(token) => token,
        Err(error) => return Err(DefaultError::Other(anyhow!(error))),
    };

    Ok(Json::from(entities::auth::LoginResponse { token }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{Router, body::Body, http, http::Request};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    use crate::{
        api::{ApiRouterBuilder, State},
        entities,
        infra::repositories,
    };

    #[tokio::test]
    async fn test_login_handler_ok() {
        let messages_repository = repositories::messages::MockMessagesRepositoryTrait::default();
        let state = State {
            messages_repository: Arc::new(messages_repository),
        };
        let app = Router::from(ApiRouterBuilder::new().with_state(Arc::from(state)).build());

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/api/v1/login")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let login_response: entities::auth::LoginResponse = serde_json::from_slice(&body).unwrap();

        assert!(!login_response.token.is_empty());
    }
}
