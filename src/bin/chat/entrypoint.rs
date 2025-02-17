use std::sync::Arc;

use anyhow::anyhow;

use app::api;
use app::config::Config;
use app::core_utils::closer::Closer;
use app::core_utils::http_server::Server;
use app::core_utils::postgres_pool;
use app::infra::repositories;

pub struct Entrypoint<'a> {
    config: Config,
    closer: Closer<'a>,
}

impl<'a> Entrypoint<'_> {
    pub fn new(config: Config) -> Self {
        Self {
            config,
            closer: Closer::new(),
        }
    }

    pub async fn bootstrap_server(&mut self) -> anyhow::Result<()> {
        // todo: init logger, tracer, etc

        let pool = postgres_pool::build_pool_from_config(
            self.config.server.client_id.clone(),
            self.config.postgres.clone(),
        )
        .map_err(|err| anyhow!("failed to create pool: {:?}", err))?;

        let handler = Arc::new(api::Handler {
            messages_repository: repositories::MessagesRepository::new(pool.clone()),
        });

        self.closer.push(Box::new(move || pool.close()));

        let srv = Server::new(self.config.server.clone());
        srv.with_router(api::get_router(handler))
            .run()
            .await
            .map_err(|err| anyhow!("handling server error: {}", err))?;

        Ok(())
    }

    pub async fn shutdown(&mut self) {
        self.closer.close();
    }
}
