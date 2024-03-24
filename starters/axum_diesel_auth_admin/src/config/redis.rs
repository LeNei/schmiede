use anyhow::{Context, Result};
use redis::Client;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct RedisSettings {
    pub host: String,
}

impl RedisSettings {
    pub fn get_client(&self) -> Result<Client> {
        let url = format!("redis://{}", self.host);
        Client::open(url).context("Failed to connect to Redis")
    }
}
