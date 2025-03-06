use std::env;

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp;
use opentelemetry_sdk::{
    Resource,
    propagation::TraceContextPropagator,
    trace::{Sampler, SdkTracerProvider},
};
use tracing::Span;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, prelude::*, util::SubscriberInitExt};

const DEFAULT_OTEL_SAMPLING_RATIO: &str = "1.0";

pub struct Observability {
    tracer_provider: SdkTracerProvider,
}

impl Observability {
    pub fn setup() -> Self {
        let tracer_provider = init_tracer_provider();
        let telemetry_layer = tracing_opentelemetry::layer()
            .with_tracer(tracer_provider.tracer(env!("CARGO_PKG_NAME")))
            .with_filter(get_otel_filter());

        let fmt_layer = tracing_subscriber::fmt::layer()
            .json()
            .flatten_event(true)
            .with_level(true)
            .with_line_number(true);

        tracing_subscriber::registry()
            .with(telemetry_layer)
            .with(get_tracing_filter())
            .with(fmt_layer)
            .init();

        Self { tracer_provider }
    }

    pub fn unset(&self) {
        let _ = self.tracer_provider.shutdown().map_err(|err| {
            tracing::error!("Failed to shutdown tracer: {:?}", err);
        });
    }
}

fn init_tracer_provider() -> SdkTracerProvider {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_http()
        .build()
        .unwrap();

    let sampler = if env::var("OTEL_SAMPLING_RATIO")
        .unwrap_or_else(|_| DEFAULT_OTEL_SAMPLING_RATIO.to_string())
        .parse::<f64>()
        .unwrap()
        < 1.0
    {
        Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
            env::var("OTEL_SAMPLING_RATIO").unwrap().parse().unwrap(),
        )))
    } else {
        Sampler::AlwaysOn
    };

    let provider = SdkTracerProvider::builder()
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .with_resource(
            Resource::builder()
                .with_service_name(env!("CARGO_PKG_NAME"))
                .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
                .build(),
        )
        .build();

    global::set_tracer_provider(provider.clone());
    provider
}

fn get_otel_filter() -> EnvFilter {
    EnvFilter::new("info")
        .add_directive(
            format!("{}=debug", env!("CARGO_CRATE_NAME"))
                .parse()
                .unwrap(),
        )
        .add_directive("axum=off".parse().unwrap())
        .add_directive("hyper=off".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap())
        .add_directive("tonic=off".parse().unwrap())
        .add_directive("h2=off".parse().unwrap())
        .add_directive("reqwest=off".parse().unwrap())
}

fn get_tracing_filter() -> EnvFilter {
    EnvFilter::new("info")
        .add_directive(
            format!("{}=debug", env!("CARGO_CRATE_NAME"))
                .parse()
                .unwrap(),
        )
        .add_directive("tower_http=error".parse().unwrap())
        .add_directive("axum::rejection=trace".parse().unwrap())
        .add_directive("tokio_postgres=error".parse().unwrap())
        .add_directive("opentelemetry=off".parse().unwrap())
}

pub fn span_ok(span: &Span) {
    span.record("otel.status_code", "OK");
}

pub fn span_error(span: &Span, description: String) {
    span.record("otel.status_code", "ERROR");
    span.record("otel.status_message", description);
}
