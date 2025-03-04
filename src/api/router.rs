use std::sync::Arc;

use axum::Extension;
use tower::ServiceBuilder;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{api, api::State};

pub struct ApiRouter {
    state: Option<Arc<State>>,
}

impl ApiRouter {
    crate::self_method!(state, Arc<State>);

    pub fn new() -> Self {
        Self { state: None }
    }

    pub fn build(&self) -> OpenApiRouter {
        let mut router = OpenApiRouter::new().nest(
            "/api/v1",
            OpenApiRouter::new()
                .routes(routes!(api::v1::login::login_handler))
                .routes(routes!(api::v1::list_messages::list_messages_handler))
                .routes(routes!(api::v1::post_message::post_message_handler)),
        );

        if let Some(state) = &self.state {
            router = router.layer(ServiceBuilder::new().layer(Extension(state.clone())));
        }

        router
    }
}

impl Default for ApiRouter {
    fn default() -> Self {
        Self::new()
    }
}
