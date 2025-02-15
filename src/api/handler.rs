use crate::infra::repositories::MessagesRepository;

#[derive(Clone)]
pub struct Handler {
    pub messages_repository: MessagesRepository,
}
