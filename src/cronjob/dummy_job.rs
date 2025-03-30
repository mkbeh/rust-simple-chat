use std::{
    sync::{Arc, OnceLock},
    time,
};

use async_trait::async_trait;
use caslex::server::Process;
use tokio_util::sync::CancellationToken;

use crate::infra::repositories::messages::MessagesRepositoryTrait;

pub struct DummyProcess {
    pub ps_num: usize,
    pub messages_repository: Arc<dyn MessagesRepositoryTrait>,
}

impl DummyProcess {
    pub fn new(
        ps_num: usize,
        messages_repository: Arc<dyn MessagesRepositoryTrait>,
    ) -> &'static Self {
        static INSTANCE: OnceLock<DummyProcess> = OnceLock::new();
        INSTANCE.get_or_init(|| DummyProcess {
            ps_num,
            messages_repository,
        })
    }
}

#[async_trait]
impl Process for DummyProcess {
    async fn pre_run(&self) -> anyhow::Result<()> {
        tracing::info!("successfully pre run process #{}", self.ps_num);
        Ok(())
    }

    async fn run(&self, token: CancellationToken) -> anyhow::Result<()> {
        const DELAY_SECS: time::Duration = time::Duration::from_secs(30);
        const OFFSET: i64 = 0;
        const LIMIT: i64 = 5;

        loop {
            tokio::select! {
                _ = token.cancelled() => {
                    tracing::info!("process: #{} successfully stopped", self.ps_num);
                    return Ok(());
                }
                _ = tokio::time::sleep(DELAY_SECS) => {
                    match self.messages_repository.list_messages(OFFSET, LIMIT).await {
                        Ok(messages) => {
                            tracing::info!("messages: {:?}", messages);
                        }
                        Err(e) => {
                            tracing::error!("process: #{}, dummy job error: {:?}", self.ps_num, e);
                        }
                    }
                }
            }
        }
    }
}
