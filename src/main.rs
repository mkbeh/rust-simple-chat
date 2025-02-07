mod api;
mod entities;
mod server;
mod config;

#[tokio::main]
async fn main() {
    // todo: parse envs / app config

    let srv = server::Server::new();
    srv.bootstrap().await;
}
