use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Validate, ToSchema)]
pub struct PostMessageRequest {
    #[validate(length(min = 1, max = 300))]
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct PostMessageResponse {
    pub(crate) message_id: i64,
}

#[derive(Debug, Serialize, Deserialize, ToSchema)]
pub struct MessageResponse {
    pub message_id: i64,
    pub content: String,
    pub posted_at: DateTime<Utc>,
}
