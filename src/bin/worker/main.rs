extern crate rust_simple_chat as app;

mod entrypoint;

use app::libs;
use entrypoint::Entrypoint;

#[tokio::main]
async fn main() {
    libs::hooks::setup_panic_hook();
    libs::observability::setup_opentelemetry();
    libs::closer::push_callback(Box::new(libs::observability::unset_opentelemetry));

    let mut entry = Entrypoint::new();
    let entry_result = entry.bootstrap_server().await;

    libs::closer::cleanup_resources();

    match entry_result {
        Ok(_) => std::process::exit(0),
        Err(e) => {
            tracing::error!("Failed to start server: {}", e);
            std::process::exit(1);
        }
    }
}
