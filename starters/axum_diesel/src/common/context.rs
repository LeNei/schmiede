use super::response::ErrorResponse;
use crate::config::auth::AuthSettings;
use crate::config::database::PgPool;
use diesel_async::pooled_connection::deadpool::Object;
use diesel_async::AsyncPgConnection;
use redis::Client;

#[derive(Clone)]
pub struct ApiContext {
    pub db: PgPool,
    pub redis_client: Client,
    pub auth_settings: AuthSettings,
}

pub type Connection = Object<AsyncPgConnection>;

impl ApiContext {
    pub async fn get_db_connection(&self) -> Result<Connection, ErrorResponse> {
        self.db.get().await.map_err(|e| {
            tracing::error!("Failed to get db connection: {}", e);
            ErrorResponse::custom_error_message("Error with database connection")
        })
    }
}
