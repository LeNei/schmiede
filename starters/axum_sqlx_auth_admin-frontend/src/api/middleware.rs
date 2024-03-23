use super::helpers::verify_jwt_token;
use crate::common::context::ApiContext;
use crate::common::models::Users;
use crate::common::response::ErrorResponse;
use axum::{
    body::Body,
    extract::State,
    http::{header, Request, StatusCode},
    middleware::Next,
    response::IntoResponse,
};
use serde::Serialize;

#[derive(Debug, Serialize, Clone)]
pub struct JWTAuthMiddleware {
    pub user: Users,
}

pub async fn jwt_auth(
    State(data): State<ApiContext>,
    mut req: Request<Body>,
    next: Next,
) -> Result<impl IntoResponse, ErrorResponse> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let access_token = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|auth_header| auth_header.to_str().ok())
        .and_then(|auth_value| auth_value.strip_prefix("Bearer "))
        .map(|token| token.to_owned());

    let access_token = access_token.ok_or_else(|| {
        ErrorResponse::custom_error(
            StatusCode::UNAUTHORIZED,
            "You need to login with a valid token",
        )
    })?;

    let access_token_details = match verify_jwt_token(
        data.auth_settings.access_token_public_key.to_owned(),
        &access_token,
    ) {
        Ok(token_details) => token_details,
        Err(_) => {
            return Err(ErrorResponse::custom_error(
                StatusCode::UNAUTHORIZED,
                "You need to login with a valid token",
            ));
        }
    };

    let user_id = access_token_details.user_id;
    let mut conn = data.get_db_connection().await?;

    let user = users
        .find(user_id)
        .select(Users::as_select())
        .first(&mut conn)
        .await
        .map_err(|_| {
            ErrorResponse::custom_error(
                StatusCode::UNAUTHORIZED,
                "You need to login with a valid token",
            )
        })?;

    req.extensions_mut().insert(JWTAuthMiddleware { user });
    Ok(next.run(req).await)
}
