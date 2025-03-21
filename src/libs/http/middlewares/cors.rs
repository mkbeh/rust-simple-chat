use http::{HeaderValue, Method};
use tower_http::cors::CorsLayer;

pub fn init_cors_layer() -> CorsLayer {
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
        .allow_headers([http::header::CONTENT_TYPE])
}
