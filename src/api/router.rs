use std::sync::Arc;

use axum::routing::{get, post};
use axum::Extension;
use axum::Router;
use tower::ServiceBuilder;

use crate::api;
use crate::api::{healthz, Handler};

pub fn get_router(handler: Arc<Handler>) -> Router {
    let router = get_default_router().nest(
        "/api/v1",
        Router::new()
            .route("/messages", post(api::v1::post_message_handler))
            .route("/messages", get(api::v1::list_messages_handler))
            .layer(ServiceBuilder::new().layer(Extension(handler))),
    );

    router
}

pub fn get_default_router() -> Router {
    Router::new()
        .route("/readiness", get(healthz))
        .route("/liveness", get(healthz))
}
