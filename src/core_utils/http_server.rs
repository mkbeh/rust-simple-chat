use std::net::SocketAddr;

use anyhow::anyhow;
use axum::routing::get;
use axum::Router;
use clap::Parser;
use tokio::signal;

use crate::core_utils::healthz;

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[arg(long, env = "CLIENT_ID")]
    pub client_id: String,
    #[arg(long, env = "SERVER_HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, env = "SERVER_PORT", default_value = "8000")]
    port: String,
}

impl Config {
    pub fn get_addr(self) -> String {
        format!("{}:{}", self.host, self.port)
    }
}

pub struct Server {
    addr: String,
    router: Option<Router>,
}

impl Server {
    pub fn new(cfg: Config) -> Self {
        Server {
            addr: cfg.get_addr(),
            router: None,
        }
    }

    pub fn with_router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let router = self.router.clone().unwrap_or_else(|| get_default_router());

        let listener = tokio::net::TcpListener::bind(self.addr.clone())
            .await
            .map_err(|e| anyhow!("failed to bind to address: {}", e))?;

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| anyhow!("failed to start server: {}", e))?;

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

pub fn get_default_router() -> Router {
    Router::new()
        .route("/readiness", get(healthz))
        .route("/liveness", get(healthz))
}
