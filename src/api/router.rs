use crate::api;
use axum::routing::{get, post};
use axum::Router;

use crate::api::healthz;

pub fn get_router() -> Router {
    let router = Router::new()
        .route("/readiness", get(healthz))
        .route("/liveness", get(healthz))
        .nest(
            "/api/v1",
            Router::new()
                .route("/messages", post(api::v1::post_message_handler))
                .route("/messages", get(api::v1::list_messages_handler)),
        );
    router
}
