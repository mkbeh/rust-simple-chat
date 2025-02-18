use std::sync::Arc;

use crate::api::Handler;
use crate::core_utils::errors::ServerError;
use crate::entities;
use axum::{Extension, Json};

pub async fn list_messages_handler(
    Extension(state): Extension<Arc<Handler>>,
) -> Result<Json<Vec<entities::message::MessageResponse>>, ServerError> {
    // todo: add query params limit offset
    let result = state.messages_repository.list_messages(0, 100).await;

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
