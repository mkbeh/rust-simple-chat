use std::{env, sync::OnceLock};

use opentelemetry::{KeyValue, global, trace::TracerProvider};
use opentelemetry_otlp::{Protocol, SpanExporter, WithExportConfig};
use opentelemetry_sdk::{
    Resource,
    metrics::SdkMeterProvider,
    propagation::TraceContextPropagator,
    trace::{Sampler, SdkTracerProvider},
};
use prometheus::Registry;
use tracing_subscriber::{EnvFilter, prelude::*};

pub fn get_registry() -> Registry {
    static INSTANCE: OnceLock<Registry> = OnceLock::new();
    INSTANCE.get_or_init(Registry::new).clone()
}

fn get_tracer_provider() -> SdkTracerProvider {
    static INSTANCE: OnceLock<SdkTracerProvider> = OnceLock::new();
    INSTANCE.get_or_init(init_traces).clone()
}

fn get_meter_provider() -> SdkMeterProvider {
    static INSTANCE: OnceLock<SdkMeterProvider> = OnceLock::new();
    INSTANCE.get_or_init(init_metrics).clone()
}

fn get_resource() -> Resource {
    static RESOURCE: OnceLock<Resource> = OnceLock::new();
    RESOURCE
        .get_or_init(|| {
            Resource::builder()
                .with_service_name(env!("CARGO_PKG_NAME"))
                .with_attribute(KeyValue::new("service.version", env!("CARGO_PKG_VERSION")))
                .build()
        })
        .clone()
}

fn init_traces() -> SdkTracerProvider {
    const DEFAULT_SAMPLE_RATIO: f64 = 1.0;

    let exporter = SpanExporter::builder()
        .with_http()
        .with_protocol(Protocol::HttpBinary)
        .build()
        .expect("Failed to create span exporter");

    let ratio = env::var("OTEL_SAMPLING_RATIO")
        .unwrap_or_else(|_| DEFAULT_SAMPLE_RATIO.to_string())
        .parse::<f64>()
        .expect("Invalid OTEL_SAMPLING_RATIO");

    let sampler = if ratio < DEFAULT_SAMPLE_RATIO {
        Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(ratio)))
    } else {
        Sampler::AlwaysOn
    };

    SdkTracerProvider::builder()
        .with_resource(get_resource())
        .with_batch_exporter(exporter)
        .with_sampler(sampler)
        .build()
}

fn init_metrics() -> SdkMeterProvider {
    let exporter = opentelemetry_prometheus::exporter()
        .with_registry(get_registry())
        .build()
        .unwrap();

    SdkMeterProvider::builder()
        .with_reader(exporter)
        .with_resource(get_resource())
        .build()
}

pub fn setup_opentelemetry() -> SdkTracerProvider {
    global::set_text_map_propagator(TraceContextPropagator::new());
    let tracer_provider = get_tracer_provider();
    // Set the global tracer provider using a clone of the tracer_provider.
    // Setting global tracer provider is required if other parts of the application
    // uses global::tracer() or global::tracer_with_version() to get a tracer.
    // Cloning simply creates a new reference to the same tracer provider. It is
    // important to hold on to the tracer_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_tracer_provider(tracer_provider.clone());

    // Create a new opentelemetry layer
    let otel_layer =
        tracing_opentelemetry::layer().with_tracer(tracer_provider.tracer(env!("CARGO_PKG_NAME")));

    // For the OpenTelemetry layer, add a tracing filter to filter events from
    // OpenTelemetry and its dependent crates (opentelemetry-otlp uses crates
    // like reqwest/tonic etc.) from being sent back to OTel itself, thus
    // preventing infinite telemetry generation. The filter levels are set as
    // follows:
    // - Allow `info` level and above by default.
    // - Restrict `opentelemetry`, `hyper`, `tonic`, and `reqwest` completely.
    // Note: This will also drop events from crates like `tonic` etc. even when
    // they are used outside the OTLP Exporter. For more details, see:
    // https://github.com/open-telemetry/opentelemetry-rust/issues/761
    let filter_otel = EnvFilter::new("info")
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
        .add_directive("reqwest=off".parse().unwrap());
    let otel_layer = otel_layer.with_filter(filter_otel);

    // Create a new tracing::Fmt layer to print the logs to stdout. It has a
    // default filter of `info` level and above, and `debug` and above for logs
    // from OpenTelemetry crates. The filter levels can be customized as needed.
    let filter_fmt = EnvFilter::new("info")
        .add_directive(
            format!("{}=debug", env!("CARGO_CRATE_NAME"))
                .parse()
                .unwrap(),
        )
        .add_directive("tower_http=error".parse().unwrap())
        .add_directive("axum::rejection=trace".parse().unwrap())
        .add_directive("tokio_postgres=error".parse().unwrap())
        .add_directive("opentelemetry=error".parse().unwrap());
    let fmt_layer = tracing_subscriber::fmt::layer()
        .with_thread_names(true)
        .json()
        .flatten_event(true)
        .with_level(true)
        .with_line_number(true)
        .with_filter(filter_fmt);

    // Initialize the tracing subscriber with the OpenTelemetry layer and the
    // Fmt layer.
    tracing_subscriber::registry()
        .with(otel_layer)
        .with(fmt_layer)
        .init();

    // At this point Logs (OTel Logs and Fmt Logs) are initialized, which will
    // allow internal-logs from Tracing/Metrics initializer to be captured.

    let meter_provider = get_meter_provider();
    // Set the global meter provider using a clone of the meter_provider.
    // Setting global meter provider is required if other parts of the application
    // uses global::meter() or global::meter_with_version() to get a meter.
    // Cloning simply creates a new reference to the same meter provider. It is
    // important to hold on to the meter_provider here, so as to invoke
    // shutdown on it when application ends.
    global::set_meter_provider(meter_provider.clone());

    tracer_provider
}

pub fn unset_opentelemetry() {
    if let Err(e) = get_tracer_provider().shutdown() {
        tracing::error!("Failed to shutdown tracer provider: {}", e);
    };

    if let Err(e) = get_meter_provider().shutdown() {
        tracing::error!("Failed to shutdown meter provider: {}", e);
    };
}
