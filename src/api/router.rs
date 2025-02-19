use std::sync::Arc;

use axum::routing::{get, post};
use axum::Extension;
use axum::Router;
use tower::ServiceBuilder;

use crate::api;
use crate::api::Handler;
use crate::core_utils::http_server;

pub fn get_router(handler: Arc<Handler>) -> Router {
    let router = http_server::get_default_router().nest(
        "/api/v1",
        Router::new()
            .route("/login", post(api::v1::login))
            .route("/messages", post(api::v1::post_message_handler))
            .route("/messages", get(api::v1::list_messages_handler))
            .layer(ServiceBuilder::new().layer(Extension(handler))),
    );

    router
}
