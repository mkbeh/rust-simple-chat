use std::{borrow::Cow, fmt::Debug, future::ready, iter::once, net::SocketAddr, time::Duration};

use anyhow::anyhow;
use axum::{
    Router,
    body::{Body, HttpBody},
    extract::MatchedPath,
    http,
    http::{HeaderName, HeaderValue, Method, StatusCode, header::AUTHORIZATION},
    middleware,
    response::IntoResponse,
    routing::get,
};
use clap::Parser;
use tokio::signal;
use tower_http::{
    catch_panic::CatchPanicLayer, classify::ServerErrorsFailureClass,
    compression::CompressionLayer, cors::CorsLayer, propagate_header::PropagateHeaderLayer,
    sensitive_headers::SetSensitiveRequestHeadersLayer, timeout::TimeoutLayer, trace::TraceLayer,
};
use tracing::Span;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::libs::{
    errors::ServerError,
    http::{
        CommonServerErrors,
        middlewares::{metrics_handler, panic_handler, setup_metrics_recorder},
        swagger,
    },
    observability::{span_error, span_ok},
};

#[derive(Parser, Debug, Clone)]
pub struct Config {
    #[arg(long, env = "SERVER_HOST", default_value = "127.0.0.1")]
    host: String,
    #[arg(long, env = "SERVER_PORT", default_value = "9000")]
    port: String,
    #[arg(long, env = "SERVER_METRICS_PORT", default_value = "9007")]
    metrics_port: String,
    #[arg(long, env = "SERVER_REQUEST_TIMEOUT", default_value = "10s")]
    request_timeout: humantime::Duration,
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
    request_timeout: Duration,
    metrics_addr: String,
    router: Option<OpenApiRouter>,
}

impl Server {
    crate::self_method!(router, OpenApiRouter);

    pub fn new(cfg: Config) -> Self {
        Server {
            addr: cfg.get_addr(),
            metrics_addr: cfg.get_metrics_addr(),
            router: None,
            request_timeout: cfg.request_timeout.into(),
        }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let app_router = match self.router.clone() {
            Some(router) => self.setup_router(router),
            None => Router::from(get_default_router()),
        };

        let app_server = self.bootstrap_server(self.addr.clone(), app_router);
        let metrics_server = self.bootstrap_server(self.metrics_addr.clone(), get_metrics_router());

        // disable failure in the custom panic hook when there is a panic,
        // because we can't handle the panic in the panic middleware (exit(1) trouble)
        setup_panic_hook();

        tracing::info!(
            "Starting servers: application={}, metrics={}",
            self.addr.clone(),
            self.metrics_addr.clone()
        );

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

    fn setup_router(&self, router: OpenApiRouter) -> Router {
        let _router = swagger::get_openapi_router(router.merge(get_default_router()));

        _router
            // Panic recovery handler
            .layer(CatchPanicLayer::custom(panic_handler))
            // Prometheus metrics tracker
            .route_layer(middleware::from_fn(metrics_handler))
            // Tracing
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(|request: &axum_core::extract::Request<Body>| {
                        let matched_path = request
                            .extensions()
                            .get::<MatchedPath>()
                            .map(MatchedPath::as_str);

                        tracing::info_span!(
                            "http_request",
                            otel.kind = "server",
                            otel.status_code = tracing::field::Empty,
                            otel.status_message = tracing::field::Empty,
                            http.method = ?request.method(),
                            http.uri = ?request.uri(),
                            http.path = matched_path,
                            http.status_code = tracing::field::Empty,
                            http.request_size = request.body().size_hint().lower(),
                            http.response_size = tracing::field::Empty,
                            http.request_headers = ?request.headers(),
                        )
                    })
                    .on_response(
                        |response: &axum_core::response::Response<Body>,
                         _latency: Duration,
                         span: &Span| {
                            span.record(
                                "http.status_code",
                                tracing::field::display(response.status()),
                            );
                            span.record(
                                "http.response_size",
                                tracing::field::display(response.body().size_hint().lower()),
                            );

                            match response.status().as_u16() {
                                0..=399 => {
                                    span_ok(span);
                                }
                                _ => {
                                    span_error(span, "received error response".to_string());
                                }
                            }
                        },
                    )
                    .on_failure(
                        |error: ServerErrorsFailureClass, _latency: Duration, span: &Span| {
                            span_error(span, error.to_string());
                        },
                    ),
            )
            // Request timeout
            .layer(TimeoutLayer::new(self.request_timeout))
            // Compress responses
            .layer(CompressionLayer::new())
            // Mark the `Authorization` request header as sensitive so it doesn't show in logs
            .layer(SetSensitiveRequestHeadersLayer::new(once(AUTHORIZATION)))
            // Propagate headers from requests to responses
            .layer(PropagateHeaderLayer::new(HeaderName::from_static(
                "x-request-id",
            )))
            // Cors
            .layer(init_cors_layer())
            .fallback(fallback_handler)
            .method_not_allowed_fallback(fallback_handler_405)
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

fn get_default_router() -> OpenApiRouter {
    OpenApiRouter::new()
        .routes(routes!(readiness))
        .routes(routes!(liveness))
}

fn get_metrics_router() -> Router {
    let recorder_handle = setup_metrics_recorder();
    Router::from(
        get_default_router().route("/metrics", get(move || ready(recorder_handle.render()))),
    )
}

/// readiness
#[utoipa::path(
    get,
    path = "/readiness",
    tag = "health",
    responses(
        (status = 200)
    )
)]
async fn readiness() -> (StatusCode, Cow<'static, str>) {
    (StatusCode::OK, Cow::from("OK"))
}

/// liveness
#[utoipa::path(
    get,
    path = "/liveness",
    tag = "health",
    responses(
        (status = 200)
    )
)]
async fn liveness() -> (StatusCode, Cow<'static, str>) {
    (StatusCode::OK, Cow::from("OK"))
}

async fn fallback_handler() -> impl IntoResponse {
    ServerError::ServiceError(&CommonServerErrors::MethodNotFound)
}

async fn fallback_handler_405() -> impl IntoResponse {
    ServerError::ServiceError(&CommonServerErrors::MethodNotAllowed)
}

fn setup_panic_hook() {
    std::panic::set_hook(Box::new(move |panic_info| {
        // If the panic has a source location, record it as structured fields.
        if let Some(location) = panic_info.location() {
            tracing::error!(
                message = %panic_info,
                panic.file = location.file(),
                panic.line = location.line(),
                panic.column = location.column(),
            );
        } else {
            tracing::error!(message = %panic_info);
        }
    }))
}

fn init_cors_layer() -> CorsLayer {
    CorsLayer::new()
        .allow_origin("*".parse::<HeaderValue>().unwrap())
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::DELETE,
            Method::OPTIONS,
            Method::PUT,
            Method::HEAD,
            Method::PATCH,
        ])
        .allow_headers([http::header::CONTENT_TYPE])
}
