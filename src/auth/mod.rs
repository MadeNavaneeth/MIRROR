pub mod handler;
pub mod oauth;
pub mod providers;
pub mod storage;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthConfig {
    pub client_id: String,
    pub client_secret: String,
    pub redirect_uri: String,
    pub scopes: Vec<String>,
    pub auth_url: String,
    pub token_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthState {
    pub csrf_token: String,
    pub pkce_verifier: Option<String>,
}
