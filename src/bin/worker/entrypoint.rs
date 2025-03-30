use std::sync::Arc;

use anyhow::anyhow;
use app::{cronjob::DummyProcess, infra::repositories};
use caslex::server::{Config, Process, Server};
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

        // init processes
        let dummy_process = DummyProcess::new(1, messages_repository);
        let processes: Vec<&'static dyn Process> = vec![dummy_process];

        Server::new(Config::parse())
            .processes(&processes)
            .run()
            .await
            .map_err(|err| anyhow!("handling server error: {}", err))?;

        Ok(())
    }
}
