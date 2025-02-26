use std::sync::Arc;

use axum::Extension;
use tower::ServiceBuilder;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{api, api::Handler};

pub fn get_router(handler: Arc<Handler>) -> OpenApiRouter {
    OpenApiRouter::new()
        .nest(
            "/api/v1",
            OpenApiRouter::new()
                .routes(routes!(api::v1::login::login_handler))
                .routes(routes!(api::v1::list_messages::list_messages_handler))
                .routes(routes!(api::v1::post_message::post_message_handler)),
        )
        .layer(ServiceBuilder::new().layer(Extension(handler)))
}
