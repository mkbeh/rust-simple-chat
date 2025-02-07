use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMessageRequest {
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostMessageResponse {
    pub id: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MessageResponse {
    pub text: String,
    pub posted_at: DateTime<Utc>,
}
