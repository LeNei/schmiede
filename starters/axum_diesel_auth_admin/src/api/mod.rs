mod helpers;
mod middleware;
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
    let jwt_auth_routes = Router::new()
        .route("/logout", post(user::logout))
        .route("/user", get(user::get_me))
        .route_layer(from_fn_with_state(
            api_context.clone(),
            middleware::jwt_auth,
        ));

    Router::new()
        .route("/ping", get(ping))
        .route("/register", post(user::register_user))
        .route("/login", post(user::jwt_login_user))
        .route("/refresh", get(user::refresh_access_token))
        .nest("/auth", jwt_auth_routes)
}
