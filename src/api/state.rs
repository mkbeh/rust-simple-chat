use std::sync::Arc;

use crate::infra::repositories::messages::MessagesRepositoryTrait;

#[derive(Clone)]
pub struct State {
    pub messages_repository: Arc<dyn MessagesRepositoryTrait>,
}
