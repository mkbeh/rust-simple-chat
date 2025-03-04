use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

pub fn setup_logger() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace,tokio_postgres=error",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(
            tracing_subscriber::fmt::layer()
                .json()
                .flatten_event(true)
                .with_level(true)
                .with_line_number(true),
        )
        .init()
}
