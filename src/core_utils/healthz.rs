use axum::http::StatusCode;
use std::borrow::Cow;

pub async fn healthz() -> (StatusCode, Cow<'static, str>) {
    (StatusCode::OK, Cow::from("OK"))
}
