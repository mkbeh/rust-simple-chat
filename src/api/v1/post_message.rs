use std::sync::Arc;

use axum::{http::StatusCode, Extension, Json};
use chrono::Utc;

use crate::api::Handler;
use crate::domain;
use crate::entities;

pub async fn post_message_handler(
    Extension(state): Extension<Arc<Handler>>,
    Json(payload): Json<entities::message::PostMessageRequest>,
) -> (StatusCode, Json<entities::message::PostMessageResponse>) {
    let message_id = state
        .messages_repository
        .create_message(domain::message::PostMessage {
            content: payload.text,
            user_id: 1,
            posted_at: Utc::now(),
        })
        .await
        .unwrap();

    (
        StatusCode::OK,
        Json(entities::message::PostMessageResponse { message_id }),
    )
}
