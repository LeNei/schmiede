use anyhow::Result;
use diesel_async::pooled_connection::deadpool::Pool;
use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use secrecy::{ExposeSecret, Secret};
use serde::Deserialize;
use serde_aux::field_attributes::deserialize_number_from_string;

#[derive(Deserialize, Clone)]
pub struct DatabaseSettings {
    pub username: String,
    pub password: Secret<String>,
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub host: String,
    pub database_name: String,
    pub require_ssl: bool,
}

pub type PgPool = Pool<AsyncPgConnection>;

impl DatabaseSettings {
    fn build_connection_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port,
            self.database_name
        )
    }

    pub fn get_connection_pool(&self) -> Result<PgPool> {
        // create a new connection pool with the default config
        let config = AsyncDieselConnectionManager::<diesel_async::AsyncPgConnection>::new(
            self.build_connection_string(),
        );
        let pool = Pool::builder(config).build()?;
        Ok(pool)
    }
}
