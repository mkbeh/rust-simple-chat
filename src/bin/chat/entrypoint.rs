use std::{default::Default, sync::Arc};

use anyhow::anyhow;
use app::{
    api,
    config::Config,
    infra::repositories,
    libs,
    libs::{closer::Closer, http::Server, postgres_pool},
};

pub struct Entrypoint<'a> {
    config: Config,
    closer: Closer<'a>,
    pool: Option<deadpool_postgres::Pool>,
}

impl Entrypoint<'_> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            pool: None,
            closer: Default::default(),
        }
    }

    pub async fn bootstrap_server(&mut self) -> anyhow::Result<()> {
        let observability = libs::Observability::setup();
        self.closer.push(Box::new(move || observability.unset()));

        let pool = postgres_pool::build_pool_from_config(self.config.postgres.clone())
            .await
            .map_err(|err| anyhow!("failed to create pool: {:?}", err))?;

        self.pool = Some(pool.clone());
        self.closer.push(Box::new(move || pool.clone().close()));

        let messages_repository = repositories::MessagesRepository::new(self.pool.clone().unwrap());

        let state = Arc::new(api::State {
            messages_repository: Arc::new(messages_repository),
        });
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
