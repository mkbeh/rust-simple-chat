use std::time::Duration;

use axum::{Router, body::HttpBody, extract::MatchedPath};
use axum_core::body::Body;
use tower_http::{classify::ServerErrorsFailureClass, trace::TraceLayer};
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::libs::http::extractors as http_utils;

pub fn with_trace_layer(router: Router) -> Router {
    router.layer(
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
                    http.path = matched_path,
                    http.query_params = request.uri().query(),
                    http.status_code = tracing::field::Empty,
                    http.request_size = request.body().size_hint().lower(),
                    http.response_size = tracing::field::Empty,
                    user_agent = http_utils::user_agent(request),
                    http.request_headers = ?request.headers(),
                )
            })
            .on_request(())
            .on_body_chunk(())
            .on_eos(())
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
                            span.set_attribute("otel.status_code", "OK");
                        }
                        _ => {
                            span.set_attribute("otel.status_code", "ERROR");
                            span.set_attribute(
                                "otel.status_message",
                                "received error response".to_string(),
                            );
                        }
                    }
                },
            )
            .on_failure(
                |error: ServerErrorsFailureClass, _latency: Duration, span: &Span| {
                    span.set_attribute("otel.status_code", "ERROR");
                    span.set_attribute("otel.status_message", error.to_string());
                },
            ),
    )
}
