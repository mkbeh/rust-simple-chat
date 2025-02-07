use axum::{http::StatusCode, Json};

use crate::entities;
use crate::entities::message::PostMessageRequest;

pub async fn post_message_handler(
    Json(payload): Json<PostMessageRequest>,
) -> (StatusCode, Json<entities::message::PostMessageResponse>) {
    let msg = entities::message::PostMessageResponse { id: 1 };

    println!("{:?}", payload);

    (StatusCode::OK, Json(msg))
}
