use std::sync::Arc;

use anyhow::anyhow;
use app::{api, infra::repositories};
use caslex::server::{Config, Server};
use caslex_extra::storages::postgres_pool;

pub struct Entrypoint {
    pool: Option<deadpool_postgres::Pool>,
}

impl Entrypoint {
    pub fn new() -> Self {
        Self { pool: None }
    }

    pub async fn bootstrap_server(&mut self) -> anyhow::Result<()> {
        let pool = postgres_pool::build_pool_from_config(postgres_pool::Config::parse())
            .await
            .map_err(|err| anyhow!("failed to create pool: {:?}", err))?;

        self.pool = Some(pool.clone());
        caslex_extra::closer::push_callback(Box::new(move || pool.clone().close()));

        let messages_repository = Arc::new(repositories::MessagesRepository::new(
            self.pool.clone().unwrap(),
        ));

        let state = Arc::new(api::State {
            messages_repository,
        });

        let router = api::ApiRouterBuilder::new()
            .with_state(state.clone())
            .build();

        Server::new(Config::parse())
            .router(router)
            .run()
            .await
            .map_err(|err| anyhow!("handling server error: {}", err))?;

        Ok(())
    }
}
