use axum::Router;
use deadpool_postgres::Pool;
use std::error::Error;
use std::net::SocketAddr;
use tokio::signal;

use crate::config::ServerConfig;

pub trait Closer {
    fn close(&self);
}

impl Closer for Pool {
    fn close(&self) {}
}

pub struct Server {
    addr: String,
    closers: Vec<Box<dyn Closer>>,
}

impl Server {
    pub fn new(cfg: ServerConfig) -> Self {
        Server {
            addr: cfg.get_addr(),
            closers: Vec::new(),
        }
    }

    pub async fn run(&self, router: Router) -> Result<(), Box<dyn Error>> {
        let listener = tokio::net::TcpListener::bind(self.addr.clone()).await?;
        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await?;

        Ok(())
    }

    pub async fn shutdown(&self) {
        self.closers.iter().for_each(|c| c.close());
    }

    pub fn add_closer(&mut self, closer: impl Closer + 'static) {
        self.closers.push(Box::new(closer));
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
