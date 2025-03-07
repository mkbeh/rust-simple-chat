use std::sync::Arc;

use axum::{Extension, Json};
use chrono::Utc;
use validator::Validate;

use crate::{
    api::State,
    domain, entities,
    libs::{
        errors::{AppJson, ServerError},
        jwt,
    },
};

/// Post message
///
/// Post message and save in storage.
#[utoipa::path(
    post,
    path = "/messages",
    tag = super::DOCS_MESSAGES_TAG,
    security(
        ("api_key" = [])
    ),
    request_body = entities::message::PostMessageRequest,
    responses(
            (status = 200, description = "", body = entities::message::PostMessageResponse)
    )
)]
pub async fn post_message_handler(
    claims: jwt::Claims,
    Extension(state): Extension<Arc<State>>,
    AppJson(payload): AppJson<entities::message::PostMessageRequest>,
) -> Result<Json<entities::message::PostMessageResponse>, ServerError> {
    match payload.validate() {
        Ok(_) => {}
        Err(err) => {
            return Err(ServerError::ValidationError(err));
        }
    }

    let result = state
        .messages_repository
        .create_message(domain::message::PostMessage {
            content: payload.text,
            user_id: claims.get_user_id(),
            posted_at: Utc::now(),
        })
        .await;

    let message_id = match result {
        Ok(message_id) => message_id,
        Err(err) => return Err(ServerError::DatabaseError(err)),
    };

    Ok(Json(entities::message::PostMessageResponse { message_id }))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{Router, body::Body, extract::Request};
    use http_body_util::BodyExt;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use crate::{
        api,
        api::{ApiRouter, State},
        infra::repositories,
    };

    #[tokio::test]
    async fn test_post_message_handler_ok() {
        let mut messages_repository =
            repositories::messages::MockMessagesRepositoryTrait::default();

        messages_repository
            .expect_create_message()
            .withf(|x| x.content == *"test-msg" && x.user_id == 123)
            .once()
            .returning(|_| Box::pin(async { Ok(1) }));

        let state = State {
            messages_repository: Arc::new(messages_repository),
        };
        let app = Router::from(ApiRouter::new().state(Arc::from(state)).build());

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::POST)
                    .uri("/api/v1/messages")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .header(http::header::AUTHORIZATION, api::generate_test_token())
                    .body(Body::from(
                        serde_json::to_vec(&json!({ "text": "test-msg" })).unwrap(),
                    ))
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let body_json: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(body_json, json!({"message_id": 1}));
    }
}
