use super::templates::Login;
use crate::common::context::ApiContext;
use crate::common::models::{LoginUserSchema, Users};
use crate::common::response::ErrorResponse;
use argon2::{Argon2, PasswordHash, PasswordVerifier};
use axum::{extract::State, response::IntoResponse, Form};
use axum_extra::extract::cookie::{Cookie, SameSite};
use http::{header, HeaderMap, Response, StatusCode};
use redis::AsyncCommands;
use serde_json::json;

pub async fn get_login_page() -> Login {
    Login {}
}

#[tracing::instrument(name = "Session Login", skip(data, body))]
pub async fn session_login_user(
    State(data): State<ApiContext>,
    Form(body): Form<LoginUserSchema>,
) -> Result<impl IntoResponse, ErrorResponse> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let mut conn = data.get_db_connection().await?;

    let user = users
        .filter(email.eq(body.email.to_ascii_lowercase()))
        .select(Users::as_select())
        .first(&mut conn)
        .await
        .map_err(|_| {
            ErrorResponse::custom_error(StatusCode::BAD_REQUEST, "Invalid email or password")
        })?;

    let is_valid = match PasswordHash::new(&user.password) {
        Ok(parsed_hash) => Argon2::default()
            .verify_password(body.password.as_bytes(), &parsed_hash)
            .map_or(false, |_| true),
        Err(_) => false,
    };

    if !is_valid {
        return Err(ErrorResponse::custom_error(
            StatusCode::BAD_REQUEST,
            "Invalid email or password",
        ));
    }

    let session_token = uuid::Uuid::new_v4().to_string();

    let mut redis_client = data
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            tracing::error!("Failed to save token to redis: {}", e);
            ErrorResponse::default()
        })?;

    redis_client
        .set_ex(&session_token, user.id.to_string(), 60 * 60)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save token to redis: {}", e);
            ErrorResponse::default()
        })?;

    let session_cookie = Cookie::build(("session", session_token))
        .path("/")
        .max_age(time::Duration::minutes(60 * 60))
        .same_site(SameSite::Lax);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(60 * 60))
        .same_site(SameSite::Lax);

    let mut response = Response::new(json!({"status": "success"}).to_string());

    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        session_cookie.to_string().parse().unwrap(),
    );

    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    headers.append(
        "HX-Redirect",
        "/".parse().map_err(|_| ErrorResponse::default())?,
    );

    response.headers_mut().extend(headers);

    Ok(response)
}
