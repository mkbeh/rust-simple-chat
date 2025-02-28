use std::sync::Arc;

use anyhow::anyhow;
use app::{
    api,
    config::Config,
    core_utils::{closer::Closer, http_server::Server, postgres_pool},
    infra::repositories,
};

pub struct Entrypoint<'a> {
    config: Config,
    closer: Closer<'a>,
}

impl Entrypoint<'_> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            closer: Closer::new(),
        }
    }

    pub async fn bootstrap_server(&mut self) -> anyhow::Result<()> {
        let pool = postgres_pool::build_pool_from_config(
            self.config.server.client_id.clone(),
            self.config.postgres.clone(),
        )
        .map_err(|err| anyhow!("failed to create pool: {:?}", err))?;

        let handler = Arc::new(api::Handler {
            messages_repository: Arc::new(repositories::MessagesRepository::new(pool.clone())),
        });

        self.closer.push(Box::new(move || pool.close()));

        Server::new(self.config.server.clone())
            .with_router(api::get_router(handler))
            .run()
            .await
            .map_err(|err| anyhow!("handling server error: {}", err))?;

        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.closer.close();
    }
}
