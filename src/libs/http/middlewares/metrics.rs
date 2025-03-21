use std::{clone::Clone, sync::OnceLock};

use axum::{
    body::{Bytes, HttpBody},
    extract::{MatchedPath, State},
    middleware::Next,
};
use axum_core::{
    extract::Request,
    response::{IntoResponse, Response},
};
use http::header::CONTENT_TYPE;
use http_body_util::Full;
use opentelemetry::{
    KeyValue, global,
    metrics::{Counter, Histogram},
};
use prometheus::{Encoder, TextEncoder};
use tokio::time::Instant;

use crate::libs::observability;

pub fn get_metrics_state() -> MetricsState {
    static INSTANCE: OnceLock<MetricsState> = OnceLock::new();
    INSTANCE.get_or_init(MetricsState::new).clone()
}

#[derive(Clone)]
pub struct MetricsState {
    http_counter: Counter<u64>,
    http_req_histogram: Histogram<f64>,
    http_req_body_gauge: Histogram<u64>,
    http_resp_body_gauge: Histogram<u64>,
}

impl MetricsState {
    fn new() -> MetricsState {
        let meter = global::meter(env!("CARGO_PKG_NAME"));

        Self {
            http_counter: meter
                .u64_counter("http_requests_total")
                .with_description("Total number of HTTP requests made.")
                .build(),
            http_req_histogram: meter
                .f64_histogram("http_request_duration")
                .with_unit("ms")
                .with_description("The HTTP request latencies in milliseconds.")
                .build(),
            http_req_body_gauge: meter
                .u64_histogram("http_request_size")
                .with_unit("By")
                .with_description("The metrics HTTP request sizes in bytes.")
                .build(),
            http_resp_body_gauge: meter
                .u64_histogram("http_response_size")
                .with_unit("By")
                .with_description("The metrics HTTP response sizes in bytes.")
                .build(),
        }
    }
}

pub async fn metrics_handler(
    State(state): State<MetricsState>,
    req: Request,
    next: Next,
) -> impl IntoResponse {
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
        KeyValue::new("method", method.to_string()),
        KeyValue::new("path", path),
        KeyValue::new("status", status),
    ];

    state.http_counter.add(1, &labels);
    state.http_req_histogram.record(latency, &labels);
    state.http_req_body_gauge.record(req_body_size, &labels);
    state.http_resp_body_gauge.record(resp_body_size, &labels);

    response
}

pub async fn prometheus_handler() -> impl IntoResponse {
    let mut buffer = vec![];
    let encoder = TextEncoder::new();
    let metric_families = observability::get_registry().gather();
    encoder.encode(&metric_families, &mut buffer).unwrap();

    Response::builder()
        .status(200)
        .header(CONTENT_TYPE, encoder.format_type())
        .body(Full::new(Bytes::from(buffer)))
        .unwrap()
        .into_response()
}
