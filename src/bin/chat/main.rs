extern crate rust_simple_chat as app;

mod entrypoint;

use app::config;
use entrypoint::Entrypoint;

#[tokio::main]
async fn main() {
    let config = config::Config::parse();

    let mut ep = Entrypoint::new();
    match ep.run_and_shutdown(config).await {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            println!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    }
}
