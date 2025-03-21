use std::time::Duration;

use anyhow::anyhow;
use clap::Parser;
use deadpool_postgres;
use humantime;

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[arg(long, env = "POSTGRES_HOST", default_value = "127.0.0.1")]
    /// Adds a host to the configuration.
    pub host: String,
    #[arg(long, env = "POSTGRES_PORT", default_value = "5432")]
    /// Adds a port to the configuration.
    pub port: u16,
    #[arg(long, env = "POSTGRES_USER", required = true)]
    /// Sets the user to authenticate with.
    pub user: String,
    #[arg(long, env = "POSTGRES_PASSWORD", required = true)]
    /// Sets the password to authenticate with.
    pub password: String,
    #[arg(long, env = "POSTGRES_DB", required = true)]
    /// Sets the name of the database to connect to.
    pub db: String,
    #[arg(long, env = "POSTGRES_SCHEMA", required = true)]
    pub schema: String,
    #[arg(long, env = "POSTGRES_CONNECT_TIMEOUT", default_value = "5s")]
    /// Sets the timeout applied to socket-level connection attempts.
    pub connect_timeout: humantime::Duration,
    #[arg(long, env = "POSTGRES_KEEPALIVES", default_value = "true")]
    /// Controls the use of TCP keepalive.
    pub keepalives: bool,
    #[arg(long, env = "POSTGRES_KEEPALIVES_IDLE", default_value = "30s")]
    /// Sets the amount of idle time before a keepalive packet is sent on the connection.
    pub keepalives_idle: humantime::Duration,
    #[arg(long, env = "POSTGRES_TARGET_SESSION_ATTRS", default_value = "any")]
    /// Sets the requirements of the session.
    pub target_session_attrs: String,
    #[arg(long, env = "POSTGRES_MAX_CONNECTIONS", default_value = "15")]
    /// Maximum size of the pool.
    pub max_connections: usize,
    #[arg(long, env = "POSTGRES_CREATE_TIMEOUT", default_value = "1m")]
    /// Timeout when creating a new object.
    pub create_timeout: humantime::Duration,
    #[arg(long, env = "POSTGRES_WAIT_TIMEOUT", default_value = "30s")]
    /// Timeout when waiting for a slot to become available.
    pub wait_timeout: humantime::Duration,
}

impl Config {
    fn get_target_session_attrs(&self) -> deadpool_postgres::TargetSessionAttrs {
        match self.target_session_attrs.as_str() {
            "any" => deadpool_postgres::TargetSessionAttrs::Any,
            "rw" => deadpool_postgres::TargetSessionAttrs::ReadWrite,
            _ => deadpool_postgres::TargetSessionAttrs::Any,
        }
    }
}

pub async fn build_pool_from_config(config: Config) -> anyhow::Result<deadpool_postgres::Pool> {
    let mut conn_opts = deadpool_postgres::Config::new();
    conn_opts.application_name = Some(env!("CARGO_PKG_NAME").to_string());
    conn_opts.host = Some(config.host.clone());
    conn_opts.port = Some(config.port);
    conn_opts.user = Some(config.user.clone());
    conn_opts.password = Some(config.password.clone());
    conn_opts.dbname = Some(config.db.clone());
    conn_opts.connect_timeout = Some(<humantime::Duration as Into<Duration>>::into(
        config.connect_timeout,
    ));
    conn_opts.keepalives = Some(config.keepalives);
    conn_opts.keepalives_idle = Some(<humantime::Duration as Into<Duration>>::into(
        config.keepalives_idle,
    ));
    conn_opts.target_session_attrs = Some(config.get_target_session_attrs());
    conn_opts.manager = Some(deadpool_postgres::ManagerConfig {
        recycling_method: deadpool_postgres::RecyclingMethod::Fast,
    });
    conn_opts.pool = Some(deadpool_postgres::PoolConfig {
        timeouts: deadpool_postgres::Timeouts {
            wait: Some(<humantime::Duration as Into<Duration>>::into(
                config.wait_timeout,
            )),
            create: Some(<humantime::Duration as Into<Duration>>::into(
                config.create_timeout,
            )),
            ..Default::default()
        },
        max_size: config.max_connections,
        queue_mode: Default::default(),
    });

    let pool = conn_opts
        .create_pool(
            Some(deadpool_postgres::Runtime::Tokio1),
            tokio_postgres::NoTls,
        )
        .map_err(|err| anyhow!("Failed create postgres pool {}", err))?;

    // ping db
    let _ = pool.get().await.map_err(|err| {
        anyhow!(
            "Failed get postgres connection {} addr: {}:{} db:{}",
            err,
            config.host,
            config.port,
            config.db
        )
    })?;

    Ok(pool)
}
