use std::{any::Any, time::Instant};

use axum::{
    body::HttpBody,
    extract::{MatchedPath, Request},
    middleware::Next,
    response::IntoResponse,
};
use http::Response;
use metrics_exporter_prometheus::{Matcher, PrometheusBuilder, PrometheusHandle};

use crate::core_utils::{errors::ServerError, http_server_errors::CommonServerErrors};

pub fn panic_handler(_: Box<dyn Any + Send + 'static>) -> Response<axum::body::Body> {
    ServerError::ServiceError(&CommonServerErrors::Panic).into_response()
}

pub async fn metrics_handler(req: Request, next: Next) -> impl IntoResponse {
    let start = Instant::now();

    let path = match req.extensions().get::<MatchedPath>() {
        Some(matched_path) => matched_path.as_str().to_owned(),
        _ => req.uri().path().to_owned(),
    };

    let method = req.method().clone();
    let req_body_size = req.body().size_hint().lower();

    let response = next.run(req).await;

    let latency = start.elapsed().as_secs_f64();
    let status = response.status().as_u16().to_string();
    let resp_body_size = response.body().size_hint().lower();

    let labels = [
        ("method", method.to_string()),
        ("path", path),
        ("status", status),
    ];

    metrics::counter!("http_requests_total", &labels).increment(1);
    metrics::histogram!("http_requests_duration_seconds", &labels).record(latency);
    metrics::gauge!("http_request_size_bytes", &labels).increment(req_body_size as f64);
    metrics::gauge!("http_response_size_bytes", &labels).increment(resp_body_size as f64);

    response
}

pub fn setup_metrics_recorder() -> PrometheusHandle {
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
