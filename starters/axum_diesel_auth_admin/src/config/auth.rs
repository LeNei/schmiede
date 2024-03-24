use secrecy::Secret;
use serde::Deserialize;

#[derive(Deserialize, Clone)]
pub struct AuthSettings {
    pub access_token_private_key: Secret<String>,
    pub access_token_public_key: String,
    pub refresh_token_private_key: Secret<String>,
    pub refresh_token_public_key: String,
}
