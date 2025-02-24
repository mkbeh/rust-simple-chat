use std::sync::Arc;

use axum::{Extension, Json, extract::Query};

use crate::{
    api::{Handler, query},
    core_utils::{errors::ServerError, jwt},
    entities,
};

pub async fn list_messages_handler(
    _: jwt::Claims,
    Extension(state): Extension<Arc<Handler>>,
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
