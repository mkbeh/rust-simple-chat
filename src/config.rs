use clap::Parser;

pub struct Config {
    pub server: ServerConfig,
    pub postgres: PostgresConfig,
}

impl Config {
    pub fn parse() -> Self {
        Config {
            server: ServerConfig::parse(),
            postgres: PostgresConfig::parse(),
        }
    }
}

#[derive(Parser, Debug)]
pub struct ServerConfig {
    #[arg(long, env = "SERVER_HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, env = "SERVER_PORT", default_value = "8000")]
    port: String,
}

impl ServerConfig {
    pub fn get_addr(self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

#[derive(Parser, Debug)]
pub struct PostgresConfig {
    #[arg(long, env = "DATABASE_HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, env = "DATABASE_PORT", default_value = "5432")]
    port: String,
}
