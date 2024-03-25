use crate::common::context::ApiContext;
use axum::{routing::get, Router};
use http::StatusCode;

#[tracing::instrument(name = "Ping")]
async fn ping() -> StatusCode {
    StatusCode::OK
}

pub fn routes() -> Router<ApiContext> {
    Router::new().route("/ping", get(ping))
}
