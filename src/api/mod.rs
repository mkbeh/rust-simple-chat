mod query;
pub mod router;
pub mod state;
pub mod v1;

pub use self::{router::ApiRouterBuilder, state::State};
use crate::libs::jwt;

pub fn generate_test_token() -> String {
    let token = jwt::encode_token(&jwt::Claims {
        sub: 123.to_string(),
        exp: jwt::expiry(1_000),
    })
    .unwrap();

    format!("Bearer {}", token)
}
