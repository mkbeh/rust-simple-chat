extern crate rust_simple_chat as chat;

use chat::api;
use chat::config;
use chat::server;

#[tokio::main]
async fn main() {
    let config = config::Config::parse();
    let router = api::get_router();

    let srv = server::Server::new(config.server);
    match srv.run(router).await {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            println!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    }
}
