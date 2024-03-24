use super::helpers::{generate_jwt_token, save_token_data_to_redis, verify_jwt_token};
use super::middleware::JWTAuthMiddleware;
use crate::common::context::ApiContext;
use crate::common::models::{LoginUserSchema, NewUser, RegisterUserSchema, Users};
use crate::common::response::ErrorResponse;
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use axum::{
    extract::State,
    response::{IntoResponse, Response},
    Extension, Json,
};
use axum_extra::extract::{
    cookie::{Cookie, SameSite},
    CookieJar,
};
use http::{header, HeaderMap, StatusCode};
use rand_core::OsRng;
use redis::AsyncCommands;
use secrecy::ExposeSecret;
use serde_json::json;

#[tracing::instrument(name = "Register new user", skip(data, body))]
pub async fn register_user(
    State(data): State<ApiContext>,
    Json(body): Json<RegisterUserSchema>,
) -> Result<(StatusCode, Json<Users>), ErrorResponse> {
    use crate::schema::users::dsl::*;
    use diesel::dsl::exists;
    use diesel::prelude::*;
    use diesel::select;
    use diesel_async::RunQueryDsl;

    let mut conn = data.get_db_connection().await?;

    let user_exists: bool = select(exists(
        users.filter(email.eq(body.email.to_owned().to_ascii_lowercase())),
    ))
    .get_result(&mut conn)
    .await
    .map_err(|e| {
        tracing::error!("Failed to fetch user exists: {}", e);
        ErrorResponse::default()
    })?;

    if user_exists {
        return Err(ErrorResponse::custom_error(
            StatusCode::CONFLICT,
            "User with that email already exists",
        ));
    }

    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
        .hash_password(body.password.as_bytes(), &salt)
        .map_err(|e| {
            tracing::error!("Failed to hash password: {}", e);
            ErrorResponse::custom_error_message("Error while hashing password")
        })
        .map(|hash| hash.to_string())?;

    let new_user = NewUser {
        first_name: &body.first_name,
        last_name: &body.last_name,
        email: &body.email,
        password: &hashed_password,
        role: "user",
        verified: false,
    };

    let user = diesel::insert_into(users)
        .values(&new_user)
        .returning(Users::as_returning())
        .get_result(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Failed to insert user: {}", e);
            ErrorResponse::default()
        })?;

    Ok((StatusCode::CREATED, Json(user)))
}

#[tracing::instrument(name = "JWT Login", skip(data, body))]
pub async fn jwt_login_user(
    State(data): State<ApiContext>,
    Json(body): Json<LoginUserSchema>,
) -> Result<impl IntoResponse, ErrorResponse> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let mut conn = data.get_db_connection().await?;

    let user = users
        .filter(email.eq(body.email.to_owned().to_ascii_lowercase()))
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

    let access_token_details = generate_jwt_token(
        user.id,
        &user.role,
        16,
        data.auth_settings
            .access_token_private_key
            .expose_secret()
            .to_owned(),
    )
    .map_err(|e| {
        tracing::error!("Failed to generate access token: {}", e);
        ErrorResponse::default()
    })?;
    let refresh_token_details = generate_jwt_token(
        user.id,
        &user.role,
        61,
        data.auth_settings
            .refresh_token_private_key
            .expose_secret()
            .to_owned(),
    )
    .map_err(|e| {
        tracing::error!("Failed to generate refresh token: {}", e);
        ErrorResponse::default()
    })?;

    save_token_data_to_redis(&data.redis_client, &refresh_token_details, 61)
        .await
        .map_err(|e| {
            tracing::error!("Failed to save token to redis: {}", e);
            ErrorResponse::default()
        })?;

    let refresh_cookie = Cookie::build((
        "refresh_token",
        refresh_token_details.token.unwrap_or_default(),
    ))
    .path("/")
    .max_age(time::Duration::minutes(61 * 60))
    .same_site(SameSite::Lax)
    .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(61 * 60))
        .same_site(SameSite::Lax);

    let mut response = Response::new(
        json!({"status": "success", "access_token": access_token_details.token.unwrap()})
            .to_string(),
    );

    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );

    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    response.headers_mut().extend(headers);

    Ok(response)
}

#[tracing::instrument(name = "Refresh access token", skip(data, cookie_jar))]
pub async fn refresh_access_token(
    cookie_jar: CookieJar,
    State(data): State<ApiContext>,
) -> Result<impl IntoResponse, ErrorResponse> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| {
            ErrorResponse::custom_error(StatusCode::FORBIDDEN, "could not refresh access token")
        })?;

    let refresh_token_details = match verify_jwt_token(
        data.auth_settings.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(_) => {
            return Err(ErrorResponse::custom_error(
                StatusCode::UNAUTHORIZED,
                "could not refresh access token",
            ));
        }
    };

    let mut redis_client = data
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            tracing::error!("Failed to get redis connection: {}", e);
            ErrorResponse::default()
        })?;

    let redis_token_user_id = redis_client
        .get::<_, String>(refresh_token_details.token_uuid.to_string())
        .await
        .map_err(|_| {
            ErrorResponse::custom_error(
                StatusCode::UNAUTHORIZED,
                "Token is invalid or session has expired",
            )
        })?;

    let user_id_uuid = uuid::Uuid::parse_str(&redis_token_user_id).map_err(|_| {
        ErrorResponse::custom_error(
            StatusCode::UNAUTHORIZED,
            "Token is invalid or session has expired",
        )
    })?;

    let mut conn = data.get_db_connection().await?;

    let user = users
        .find(user_id_uuid)
        .select(Users::as_select())
        .first(&mut conn)
        .await
        .map_err(|_| {
            ErrorResponse::custom_error(
                StatusCode::UNAUTHORIZED,
                "The user belonging to this token no longer exists",
            )
        })?;

    let access_token_details = generate_jwt_token(
        user.id,
        &user.role,
        16,
        data.auth_settings
            .access_token_private_key
            .expose_secret()
            .to_owned(),
    )
    .map_err(|e| {
        tracing::error!("Failed to generate access token: {}", e);
        ErrorResponse::default()
    })?;

    let response = Response::new(
        json!({"status": "success", "access_token": access_token_details.token.unwrap()})
            .to_string(),
    );

    Ok(response)
}

#[tracing::instrument(name = "Logout", skip(data, cookie_jar))]
pub async fn logout(
    cookie_jar: CookieJar,
    Extension(auth_guard): Extension<JWTAuthMiddleware>,
    State(data): State<ApiContext>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let refresh_token = cookie_jar
        .get("refresh_token")
        .map(|cookie| cookie.value().to_string())
        .ok_or_else(|| {
            ErrorResponse::custom_error(
                StatusCode::FORBIDDEN,
                "Token is invalid or session has expired",
            )
        })?;

    let refresh_token_details = match verify_jwt_token(
        data.auth_settings.refresh_token_public_key.to_owned(),
        &refresh_token,
    ) {
        Ok(token_details) => token_details,
        Err(e) => {
            tracing::error!("Failed to verify refresh token: {}", e);
            return Err(ErrorResponse::custom_error(
                StatusCode::UNAUTHORIZED,
                "Token is invalid or session has expired",
            ));
        }
    };

    let mut redis_client = data
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            tracing::error!("Failed to get redis connection: {}", e);
            ErrorResponse::default()
        })?;

    redis_client
        .del(&[refresh_token_details.token_uuid.to_string()])
        .await
        .map_err(|e| {
            tracing::error!("Failed to delete token from redis: {}", e);
            ErrorResponse::default()
        })?;

    let refresh_cookie = Cookie::build(("refresh_token", ""))
        .path("/")
        .max_age(time::Duration::minutes(0))
        .same_site(SameSite::Lax)
        .http_only(true);

    let logged_in_cookie = Cookie::build(("logged_in", "true"))
        .path("/")
        .max_age(time::Duration::minutes(0))
        .same_site(SameSite::Lax);

    let mut headers = HeaderMap::new();
    headers.append(
        header::SET_COOKIE,
        refresh_cookie.to_string().parse().unwrap(),
    );
    headers.append(
        header::SET_COOKIE,
        logged_in_cookie.to_string().parse().unwrap(),
    );

    let mut response = Response::new(json!({"status": "success"}).to_string());
    response.headers_mut().extend(headers);
    Ok(response)
}

pub async fn get_me(
    Extension(jwtauth): Extension<JWTAuthMiddleware>,
) -> Result<Json<Users>, ErrorResponse> {
    Ok(Json(jwtauth.user))
}
