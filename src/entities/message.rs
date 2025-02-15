use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMessageRequest {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMessageResponse {
    pub(crate) message_id: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub message_id: i64,
    pub content: String,
    pub posted_at: DateTime<Utc>,
}
