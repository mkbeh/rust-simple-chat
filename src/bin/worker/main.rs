extern crate rust_simple_chat as app;

mod entrypoint;

use app::{config::Config, libs};
use entrypoint::Entrypoint;

#[tokio::main]
async fn main() {
    libs::hooks::setup_panic_hook();

    let config = Config::parse();
    let mut entry = Entrypoint::new(config);

    let result = entry.bootstrap_server().await;
    entry.shutdown().await;

    match result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            tracing::error!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    }
}
