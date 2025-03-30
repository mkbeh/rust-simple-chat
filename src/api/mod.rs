mod query;
pub mod router;
pub mod state;
pub mod v1;

pub use self::{router::ApiRouterBuilder, state::State};

pub fn generate_test_token() -> String {
    use caslex::middlewares::auth;
    use caslex_extra::security::jwt;

    let token = jwt::encode_token(&auth::Claims {
        sub: 123.to_string(),
        exp: jwt::expiry(1_000),
    })
    .unwrap();

    format!("Bearer {}", token)
}
