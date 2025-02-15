use std::sync::Arc;

use anyhow::anyhow;

use app::api;
use app::config::Config;
use app::infra::repositories;
use app::server::Server;

pub struct Entrypoint {}

impl Entrypoint {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run_and_shutdown(&mut self, cfg: Config) -> anyhow::Result<()> {
        // todo: init logger, tracer, etc

        let pool = repositories::pool::build_pool_from_config(
            cfg.server.client_id.clone(),
            cfg.postgres.clone(),
        )
        .map_err(|err| anyhow!("failed to create pool: {:?}", err))?;

        let messages_repository = repositories::MessagesRepository::new(pool.clone());

        let handler = Arc::new(api::Handler {
            messages_repository,
        });

        let router = api::get_router(handler);
        let mut srv = Server::new(cfg.server);

        srv.run(router)
            .await
            .map_err(|err| anyhow!("handling server error: {}", err))?;

        srv.add_closer(pool);
        srv.shutdown().await;

        Ok(())
    }
}
