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

    pub async fn bootstrap(self) {
        let router = api::get_router();

        let listener = tokio::net::TcpListener::bind("127.0.0.1:8000")
            .await
            .unwrap();
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signals())
        .await
        .unwrap();
    }
}

async fn shutdown_signals() {
    loop {
        // todo
    }
}
