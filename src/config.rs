use clap::Parser;

use crate::libs::{http_server, postgres_pool};

pub struct Config {
    pub server: http_server::Config,
    pub postgres: postgres_pool::Config,
}

impl Config {
    pub fn parse() -> Self {
        Config {
            server: http_server::Config::parse(),
            postgres: postgres_pool::Config::parse(),
        }
    }
}
