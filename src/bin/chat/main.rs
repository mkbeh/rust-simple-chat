extern crate rust_simple_chat as app;

mod entrypoint;

use app::config;
use app::core_utils;
use entrypoint::Entrypoint;

#[tokio::main]
async fn main() {
    core_utils::hooks::setup_panic_hook();

    let config = config::Config::parse();

    let mut ep = Entrypoint::new(config);
    match ep.bootstrap_server().await {
        Ok(_) => {
            ep.shutdown().await;
            std::process::exit(0)
        }
        Err(e) => {
            ep.shutdown().await;
            println!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    }
}
