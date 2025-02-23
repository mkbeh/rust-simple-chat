use crate::{api, api::Handler, core_utils::http_server};
use axum::Extension;
use axum::Router;
use axum::routing::{get, post};
use std::sync::Arc;
use tower::ServiceBuilder;

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
