use std::{default::Default, sync::Arc};

use anyhow::anyhow;
use app::{
    api,
    infra::repositories,
    libs,
    libs::{closer::Closer, http, http::Server, postgres_pool},
};
use clap::Parser;

pub struct Entrypoint<'a> {
    closer: Closer<'a>,
    pool: Option<deadpool_postgres::Pool>,
}

impl Entrypoint<'_> {
    pub fn new() -> Self {
        Self {
            pool: None,
            closer: Closer::default(),
        }
    }

    pub async fn bootstrap_server(&mut self) -> anyhow::Result<()> {
        let observability = libs::Observability::setup();
        self.closer.push(Box::new(move || observability.unset()));

        let pool = postgres_pool::build_pool_from_config(postgres_pool::Config::parse())
            .await
            .map_err(|err| anyhow!("failed to create pool: {:?}", err))?;

        self.pool = Some(pool.clone());
        self.closer.push(Box::new(move || pool.clone().close()));

        let messages_repository = Arc::new(repositories::MessagesRepository::new(
            self.pool.clone().unwrap(),
        ));

        let state = Arc::new(api::State {
            messages_repository,
        });

        Server::new(http::server::Config::parse())
            .router(api::ApiRouter::new().state(state.clone()).build())
            .run()
            .await
            .map_err(|err| anyhow!("handling server error: {}", err))?;

        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.closer.close();
    }
}
