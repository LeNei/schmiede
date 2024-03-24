use super::{
    common::TablePaginationParams,
    middleware::SessionAuthMiddleware,
    templates::{Form, FormInput, FormInputType, FormPage, TablePageBuilder},
};
use crate::common::{
    context::ApiContext, models::Users, response::ErrorResponse, traits::Displayable,
};
use axum::{
    extract::{Path, Query, State},
    response::IntoResponse,
    routing::get,
    Extension, Router,
};
use diesel_async::RunQueryDsl;

pub async fn get_users(
    Query(params): Query<TablePaginationParams>,
    State(data): State<ApiContext>,
    Extension(user): Extension<SessionAuthMiddleware>,
) -> Result<impl IntoResponse, ErrorResponse> {
    let mut conn = data.get_db_connection().await?;

    let (offset, page_size, search_term) = params.to_query_params();

    let all_users = Users::paginated(offset, page_size, search_term)
        .load(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching from database: {}", e);
            ErrorResponse::custom_error_message("Error fetching from database")
        })?;

    Ok(TablePageBuilder::new()
        .title("Users")
        .description("This is a list of users")
        .add_title("Add User")
        .url("/users")
        .headers::<Users>()
        .rows(&all_users)
        .use_sidebar(!user.is_hx)
        .build())
}

async fn get_user_by_id(
    Path(user_id): Path<uuid::Uuid>,
    State(data): State<ApiContext>,
    Extension(user): Extension<SessionAuthMiddleware>,
) -> Result<impl IntoResponse, ErrorResponse> {
    use crate::schema::users::dsl::*;
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let mut conn = data.get_db_connection().await?;

    let db_user = users
        .find(user_id)
        .select(Users::as_select())
        .first(&mut conn)
        .await
        .map_err(|e| {
            tracing::error!("Error fetching user from database: {}", e);
            ErrorResponse::custom_error_message("Error fetching user from database")
        })?;

    let res = Form {
        title: "User",
        description: "This is a user",
        inputs: vec![FormInput {
            label: "First Name",
            name: "first_name",
            value: Some(db_user.first_name),
            required: true,
            input_type: FormInputType::Text,
        }],
    };

    if user.is_hx {
        return Ok(res.into_response());
    }

    Ok(FormPage::from(&res).into_response())
}

pub fn routes() -> Router<ApiContext> {
    Router::new()
        .route("/users", get(get_users))
        .route("/users/:user_id", get(get_user_by_id))
}
