use crate::libs;

pub mod closer;
pub mod hooks;
pub mod http;
pub mod jwt;
pub mod observability;
pub mod postgres_pool;

pub fn setup_application() {
    // Setup application panic hook
    hooks::setup_panic_hook();

    // Setup logs/tracing
    observability::setup_opentelemetry();
    closer::push_callback(Box::new(libs::observability::unset_opentelemetry));
}
