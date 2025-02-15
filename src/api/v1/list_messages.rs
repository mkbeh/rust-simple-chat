use std::sync::Arc;

use axum::http::StatusCode;
use axum::{Extension, Json};

use crate::api::Handler;
use crate::entities;

pub async fn list_messages_handler(
    Extension(state): Extension<Arc<Handler>>,
) -> (StatusCode, Json<Vec<entities::message::MessageResponse>>) {
    let db_messages = state
        .messages_repository
        .list_messages(0, 100)
        .await
        .unwrap();

    (
        StatusCode::OK,
        Json(
            db_messages
                .into_iter()
                .map(|msg| entities::message::MessageResponse {
                    message_id: msg.message_id,
                    content: msg.message_content,
                    posted_at: msg.posted_at,
                })
                .collect(),
        ),
    )
}
