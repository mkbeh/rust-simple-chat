use std::sync::Arc;

use anyhow::anyhow;
use app::{cronjob::DummyProcess, infra::repositories};
use rust_simple_chat::{
    config::Config,
    libs,
    libs::{
        closer::Closer,
        http::{server::Process, Server},
        postgres_pool,
    },
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
            closer: Closer::default(),
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

        // init dummy process
        let dummy_ps = DummyProcess::new(1, Arc::new(messages_repository));
        let processes: Vec<&'static dyn Process> = vec![dummy_ps];

        Server::new(self.config.server.clone())
            .processes(&processes)
            .run()
            .await
            .map_err(|err| anyhow!("handling server error: {}", err))?;

        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.closer.close();
    }
}
