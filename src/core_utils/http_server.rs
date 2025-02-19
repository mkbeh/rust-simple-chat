use anyhow::anyhow;
use axum::{
    extract::{MatchedPath, Request},
    middleware,
    middleware::Next,
    response::IntoResponse,
    routing::get,
    Router,
};
use clap::Parser;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};
use std::{future::ready, net::SocketAddr, time::Instant};
use tokio::signal;

use crate::core_utils::healthz;

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[arg(long, env = "CLIENT_ID")]
    pub client_id: String,
    #[arg(long, env = "SERVER_HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, env = "SERVER_PORT", default_value = "9000")]
    port: String,
    #[arg(long, env = "SERVER_METRICS_PORT", default_value = "9007")]
    metrics_port: String,
}

impl Config {
    fn get_addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    fn get_metrics_addr(&self) -> String {
        format!("{}:{}", self.host, self.metrics_port)
    }
}

pub struct Server {
    addr: String,
    metrics_addr: String,
    router: Option<Router>,
}

impl Server {
    pub fn new(cfg: Config) -> Self {
        Server {
            addr: cfg.get_addr(),
            metrics_addr: cfg.get_metrics_addr(),
            router: None,
        }
    }

    pub fn with_router(mut self, router: Router) -> Self {
        self.router = Some(router);
        self
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let app_router = self.router.clone().unwrap_or_else(|| get_default_router());
        let app_router = Self::add_metrics_middleware(app_router);
        let app_server = self.bootstrap_server(self.addr.clone(), app_router);

        let metrics_router = get_metrics_router();
        let metrics_server = self.bootstrap_server(self.metrics_addr.clone(), metrics_router);

        tokio::try_join!(app_server, metrics_server)
            .map_err(|e| anyhow!("Failed to bootstrap server. Reason: {:?}", e))?;

        Ok(())
    }

    async fn bootstrap_server(&self, addr: String, router: Router) -> anyhow::Result<()> {
        let listener = tokio::net::TcpListener::bind(addr.clone())
            .await
            .map_err(|e| anyhow!("failed to bind to address: {e}"))?;

        axum::serve(
            listener,
            router.into_make_service_with_connect_info::<SocketAddr>(),
        )
        .with_graceful_shutdown(shutdown_signal())
        .await
        .map_err(|e| anyhow!("failed to start server on address {addr}: {e}"))?;

        Ok(())
    }

    fn add_metrics_middleware(router: Router) -> Router {
        router.route_layer(middleware::from_fn(track_metrics))
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

pub fn get_metrics_router() -> Router {
    let recorder_handle = setup_metrics_recorder();
    get_default_router().route("/metrics", get(move || ready(recorder_handle.render())))
}

fn setup_metrics_recorder() -> PrometheusHandle {
    const EXPONENTIAL_SECONDS: &[f64] = &[
        0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0,
    ];

    PrometheusBuilder::new()
        .set_buckets_for_metric(
            Matcher::Full("http_requests_duration_seconds".to_string()),
            EXPONENTIAL_SECONDS,
        )
        .unwrap()
        .install_recorder()
        .unwrap()
}

async fn track_metrics(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();
    let path = if let Some(matched_path) = req.extensions().get::<MatchedPath>() {
        matched_path.as_str().to_owned()
    } else {
        req.uri().path().to_owned()
    };
    let method = req.method().clone();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_requests_duration_seconds", &labels).record(latency);

    response
}
