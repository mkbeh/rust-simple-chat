use async_trait::async_trait;
use deadpool_postgres::Pool;
use mockall::*;

use crate::domain::message;

#[async_trait]
#[automock]
pub trait MessagesRepositoryTrait: Send + Sync {
    async fn create_message(&self, msg: message::PostMessage)
    -> anyhow::Result<i64, anyhow::Error>;
    async fn list_messages(
        &self,
        offset: i64,
        limit: i64,
    ) -> anyhow::Result<Vec<message::Message>, anyhow::Error>;
}

#[derive(Clone)]
pub struct MessagesRepository {
    pool: Pool,
}

impl MessagesRepository {
    pub fn new(pool: Pool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl MessagesRepositoryTrait for MessagesRepository {
    async fn create_message(
        &self,
        msg: message::PostMessage,
    ) -> anyhow::Result<i64, anyhow::Error> {
        let client = self.pool.get().await?;
        let stmt = client
            .prepare(
                // language=postgresql
                r#"
                INSERT INTO rust_simple_chat.messages (message_content, user_id, posted_at)
                VALUES ($1, $2, $3)
                RETURNING message_id AS message_id;"#,
            )
            .await?;

        let row = client
            .query_one(&stmt, &[&msg.content, &msg.user_id, &msg.posted_at])
            .await?;

        let message_id: i64 = row.get("message_id");

        Ok(message_id)
    }

    async fn list_messages(
        &self,
        offset: i64,
        limit: i64,
    ) -> anyhow::Result<Vec<message::Message>, anyhow::Error> {
        let client = self.pool.get().await?;
        let stmt = client
            .prepare_cached(
                // language=postgresql
                r#"
                SELECT message_id      AS message_id,
                       message_content AS message_content,
                       user_id         AS user_id,
                       posted_at       AS posted_at
                FROM rust_simple_chat.messages
                ORDER BY posted_at DESC
                OFFSET $1 LIMIT $2;
                "#,
            )
            .await?;

        let rows = client.query(&stmt, &[&offset, &limit]).await?;

        Ok(rows.iter().map(message::Message::from).collect())
    }
}
