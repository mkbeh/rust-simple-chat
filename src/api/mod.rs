pub mod handler;
mod query;
pub mod router;
pub mod v1;

pub use self::{handler::Handler, router::get_router};
use crate::core_utils::jwt;

pub fn generate_test_token() -> String {
    let token = jwt::encode_token(&jwt::Claims {
        sub: 123.to_string(),
        exp: jwt::expiry(1_000),
    })
    .unwrap();

    format!("Bearer {}", token)
}
