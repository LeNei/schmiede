use crate::common::context::ApiContext;
use crate::common::models::Users;
use axum::{
    body::Body,
    extract::State,
    http::Request,
    middleware::Next,
    response::{IntoResponse, Redirect},
};
use axum_extra::extract::CookieJar;
use redis::AsyncCommands;

#[derive(Clone)]
pub struct SessionAuthMiddleware {
    pub user: Users,
    pub is_hx: bool,
}

pub async fn session_auth(
    cookie_jar: CookieJar,
    State(data): State<ApiContext>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, Redirect> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let redirect = Redirect::to("/login");
    let session_cookie = cookie_jar
        .get("session")
        .ok_or_else(|| redirect.clone())?
        .value()
        .to_owned();

    let is_hx = req.headers().get("hx-request").is_some();

    let mut redis_conn = data
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            tracing::error!("Error getting redis connection: {:?}", e);
            redirect.clone()
        })?;

    let user_id = redis_conn
        .get::<_, String>(&session_cookie)
        .await
        .map_err(|_| redirect.clone())?;

    let user_id_uuid = uuid::Uuid::parse_str(&user_id).map_err(|_| redirect.clone())?;

    let mut conn = data
        .get_db_connection()
        .await
        .map_err(|_| redirect.clone())?;

    let user = users
        .find(user_id_uuid)
        .select(Users::as_select())
        .first(&mut conn)
        .await
        .map_err(|_| redirect.clone())?;

    req.extensions_mut()
        .insert(SessionAuthMiddleware { user, is_hx });
    Ok(next.run(req).await)
}
