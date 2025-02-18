use std::sync::Arc;

use axum::{Extension, Json};
use chrono::Utc;
use validator::Validate;

use crate::api::Handler;
use crate::core_utils::errors::AppJson;
use crate::core_utils::errors::ServerError;
use crate::domain;
use crate::entities;

pub async fn post_message_handler(
    Extension(state): Extension<Arc<Handler>>,
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
            user_id: 1,
            posted_at: Utc::now(),
        })
        .await;

    let message_id = match result {
        Ok(message_id) => message_id,
        Err(err) => return Err(ServerError::DatabaseError(err)),
    };

    Ok(Json(entities::message::PostMessageResponse { message_id }))
}
