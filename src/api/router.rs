use std::sync::Arc;

use axum::http::{HeaderValue, Method};
use axum::routing::{get, post};
use axum::Router;
use axum::{http, Extension};
use tower::ServiceBuilder;
use tower_http::cors::CorsLayer;

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
            .layer(
                CorsLayer::new()
                    .allow_origin("*".parse::<HeaderValue>().unwrap())
                    .allow_methods([
                        Method::GET,
                        Method::POST,
                        Method::DELETE,
                        Method::OPTIONS,
                        Method::PUT,
                        Method::HEAD,
                        Method::PATCH,
                    ])
                    .allow_headers([http::header::CONTENT_TYPE]),
            )
            .layer(ServiceBuilder::new().layer(Extension(handler))),
    );

    router
}
