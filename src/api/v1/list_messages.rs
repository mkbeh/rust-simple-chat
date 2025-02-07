use axum::http::StatusCode;
use axum::Json;

use crate::entities;

pub async fn list_messages_handler() -> (StatusCode, Json<Vec<entities::message::MessageResponse>>)
{
    let messages = vec![];
    (StatusCode::OK, Json(messages))
}
