mod common;
mod login;
mod middleware;
mod templates;
mod user;

use crate::common::context::ApiContext;
use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
    Router,
};
use http::StatusCode;

#[tracing::instrument(name = "Ping")]
async fn ping() -> StatusCode {
    StatusCode::OK
}

pub fn routes(api_context: &ApiContext) -> Router<ApiContext> {
    let session_auth_routes = Router::new()
        .route("/", get(user::get_users))
        .merge(user::routes())
        .route_layer(from_fn_with_state(
            api_context.clone(),
            middleware::session_auth,
        ));

    Router::new()
        .route("/login", get(login::get_login_page))
        .route("/login", post(login::session_login_user))
        .merge(session_auth_routes)
}
