use std::time::Duration;

use anyhow::anyhow;
use deadpool_postgres;

use crate::config::PostgresConfig;

pub fn build_pool_from_config(
    client_id: String,
    config: PostgresConfig,
) -> anyhow::Result<deadpool_postgres::Pool> {
    let mut conn_opts = deadpool_postgres::Config::new();
    conn_opts.application_name = Some(client_id);
    conn_opts.host = Some(config.host);
    conn_opts.port = Some(config.port);
    conn_opts.user = Some(config.user);
    conn_opts.password = Some(config.password);
    conn_opts.dbname = Some(config.db);
    conn_opts.manager = Some(deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Fast,
    });
    conn_opts.pool = Some(deadpool_postgres::PoolConfig {
        timeouts: deadpool_postgres::Timeouts {
            wait: Some(Duration::from(config.wait_timeout.into())),
            create: Some(Duration::from(config.create_timeout.into())),
            ..Default::default()
        },
        max_size: config.max_connections as usize,
        queue_mode: Default::default(),
    });

    let pool = conn_opts
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .map_err(|err| anyhow!("Failed connect to database {}", err))?;

    Ok(pool)
}
