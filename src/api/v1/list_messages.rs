use std::sync::Arc;

use axum::{Extension, Json, extract::Query};

use crate::{
    api::{State, query},
    entities,
    libs::{http::errors::ServerError, jwt},
};

/// List all messages
///
/// List all messages from storage.
#[utoipa::path(
    get,
    path = "/messages",
    tag = super::DOCS_MESSAGES_TAG,
    security(
        ("api_key" = [])
    ),
    params(
        query::Pagination
    ),
    responses(
        (status = 200, description = "List all messages successfully", body = [entities::message::MessageResponse])
    )
)]
pub async fn list_messages_handler(
    _: jwt::Claims,
    Extension(state): Extension<Arc<State>>,
    Query(params): Query<query::Pagination>,
) -> Result<Json<Vec<entities::message::MessageResponse>>, ServerError> {
    let result = state
        .messages_repository
        .list_messages(params.get_offset(), params.get_limit())
        .await;

    let db_messages = match result {
        Ok(db_messages) => db_messages,
        Err(err) => return Err(ServerError::DatabaseError(err)),
    };

    Ok(Json(
        db_messages
            .into_iter()
            .map(|msg| entities::message::MessageResponse {
                message_id: msg.message_id,
                content: msg.message_content,
                posted_at: msg.posted_at,
            })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use axum::{Router, body::Body, extract::Request};
    use chrono::{DateTime, Utc};
    use http_body_util::BodyExt;
    use mockall::predicate::*;
    use serde_json::{Value, json};
    use tower::ServiceExt;

    use crate::{
        api,
        api::{ApiRouterBuilder, State},
        domain,
        infra::repositories,
    };

    #[tokio::test]
    async fn test_list_messages_handler_ok() {
        let mut messages_repository =
            repositories::messages::MockMessagesRepositoryTrait::default();

        messages_repository
            .expect_list_messages()
            .with(eq(0), eq(100))
            .once()
            .returning(|_, _| {
                Box::pin(async {
                    let posted_at =
                        DateTime::parse_from_rfc3339("2020-04-12T22:10:57+02:00").unwrap();
                    let posted_at_utc = posted_at.with_timezone(&Utc);

                    Ok(vec![domain::message::Message {
                        message_id: 1,
                        message_content: "test".to_string(),
                        user_id: 123,
                        posted_at: posted_at_utc,
                    }])
                })
            });

        let state = State {
            messages_repository: Arc::new(messages_repository),
        };
        let app = Router::from(ApiRouterBuilder::new().with_state(Arc::from(state)).build());

        let response = app
            .oneshot(
                Request::builder()
                    .method(http::Method::GET)
                    .uri("/api/v1/messages")
                    .header(http::header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
                    .header(http::header::AUTHORIZATION, api::generate_test_token())
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), http::StatusCode::OK);

        let body = response.into_body().collect().await.unwrap().to_bytes();
        let messages_response: Value = serde_json::from_slice(&body).unwrap();

        assert_eq!(
            messages_response,
            json!([
               {
                  "content":"test",
                  "message_id":1,
                  "posted_at":"2020-04-12T20:10:57Z"
               }
            ])
        );
    }
}
