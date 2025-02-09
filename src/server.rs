use axum::Router;
use std::error::Error;
use std::net::SocketAddr;
use tokio::signal;

use crate::config::ServerConfig;

pub struct Server {
    addr: String,
}

impl Server {
    pub fn new(cfg: ServerConfig) -> Self {
        Server {
            addr: cfg.get_addr(),
        }
    }

    pub async fn run(self, router: Router) -> Result<(), Box<dyn Error>> {
        let listener = tokio::net::TcpListener::bind(self.addr).await?;
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await?;

        Ok(())
    }
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(unix)]
    let quit = async {
        signal::unix::signal(signal::unix::SignalKind::quit())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
        _ = quit => {},
    }
}
