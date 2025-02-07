extern crate rust_simple_chat as chat;
use chat::server;

#[tokio::main]
async fn main() {
    // todo: parse envs / app config

    let srv = server::Server::new();
    match srv.run().await {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            println!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    }
}
