use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tokio_postgres_utils::FromRow;

#[derive(Debug, Deserialize, Serialize)]
pub struct PostMessage {
    pub content: String,
    pub user_id: i32,
    pub posted_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize, Serialize, FromRow)]
pub struct Message {
    pub message_id: i64,
    pub message_content: String,
    pub user_id: i32,
    pub posted_at: DateTime<Utc>,
}
