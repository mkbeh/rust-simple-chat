extern crate rust_simple_chat as app;

mod entrypoint;

#[tokio::main]
async fn main() {
    static SERVICE_NAME: &str = env!("CARGO_PKG_NAME");
    caslex_extra::setup_application(SERVICE_NAME);

    let mut entry = entrypoint::Entrypoint::new();
    let entry_result = entry.bootstrap_server().await;

    caslex_extra::cleanup_resources();

    match entry_result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            tracing::error!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    }
}
