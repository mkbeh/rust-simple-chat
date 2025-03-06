use std::sync::Arc;

use anyhow::anyhow;
use app::{
    api,
    config::Config,
    core_utils,
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
        let observability = core_utils::observability::Observability::setup();
        self.closer.push(Box::new(move || observability.unset()));

        let pool = postgres_pool::build_pool_from_config(self.config.postgres.clone())
            .await
            .map_err(|err| anyhow!("failed to create pool: {:?}", err))?;

        let state = Arc::new(api::State {
            messages_repository: Arc::new(repositories::MessagesRepository::new(pool.clone())),
        });

        self.closer.push(Box::new(move || pool.clone().close()));

        let api_router = api::ApiRouter::new().state(state.clone()).build();

        Server::new(self.config.server.clone())
            .router(api_router)
            .run()
            .await
            .map_err(|err| anyhow!("handling server error: {}", err))?;

        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.closer.close();
    }
}
