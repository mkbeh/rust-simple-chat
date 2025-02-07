use std::error::Error;
use std::net::SocketAddr;

use crate::api;

pub struct Server {
    // todo
}

impl Server {
    pub fn new() -> Self {
        // todo
        Server {}
    }

    pub async fn run(self) -> Result<(), Box<dyn Error>> {
        let router = api::get_router();

        let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await?;
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signals())
        .await?;

        Ok(())
    }
}

async fn shutdown_signals() {
    loop {
        // todo
    }
}
