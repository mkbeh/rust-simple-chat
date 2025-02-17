use clap::Parser;
use humantime;

use crate::server;

pub struct Config {
    pub server: server::Config,
    pub postgres: PostgresConfig,
}

impl Config {
    pub fn parse() -> Self {
        Config {
            server: server::Config::parse(),
            postgres: PostgresConfig::parse(),
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub struct PostgresConfig {
    #[arg(long, env = "POSTGRES_HOST", default_value = "127.0.0.1")]
    pub host: String,
    #[arg(long, env = "POSTGRES_PORT", default_value = "5432")]
    pub port: u16,
    #[arg(long, env = "POSTGRES_USER", required = true)]
    pub user: String,
    #[arg(long, env = "POSTGRES_PASSWORD", required = true)]
    pub password: String,
    #[arg(long, env = "POSTGRES_DB", required = true)]
    pub db: String,
    #[arg(long, env = "POSTGRES_SCHEMA", required = true)]
    pub schema: String,
    #[arg(long, env = "POSTGRES_MAX_CONNECTIONS", default_value = "50")]
    pub max_connections: usize,
    #[arg(long, env = "POSTGRES_CREATE_TIMEOUT", default_value = "1m")]
    pub create_timeout: humantime::Duration,
    #[arg(long, env = "POSTGRES_WAIT_TIMEOUT", default_value = "30s")]
    pub wait_timeout: humantime::Duration,
}
