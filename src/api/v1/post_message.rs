use std::sync::Arc;

use axum::{Extension, Json};
use chrono::Utc;
use validator::Validate;

use crate::{
    api::Handler,
    core_utils::{
        errors::{AppJson, ServerError},
        jwt,
    },
    domain, entities,
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
