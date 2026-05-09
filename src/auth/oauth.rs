use super::OAuthConfig;
use crate::plugins::traits::Credentials;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: Option<u64>,
    refresh_token: Option<String>,
    scope: Option<String>,
}

pub struct OAuthFlow {
    config: OAuthConfig,
    client: reqwest::Client,
}

impl OAuthFlow {
    pub fn new(config: OAuthConfig) -> Self {
        Self {
            config,
            client: reqwest::Client::new(),
        }
    }

    /// Generate authorization URL
    pub fn get_auth_url(&self) -> (String, String) {
        // Simple implementation for now - proper one would use oauth2 crate
        let csrf_token = uuid::Uuid::new_v4().to_string();

        let url = format!(
            "{}?response_type=code&client_id={}&redirect_uri={}&scope={}&state={}",
            self.config.auth_url,
            self.config.client_id,
            urlencoding::encode(&self.config.redirect_uri),
            urlencoding::encode(&self.config.scopes.join(" ")),
            csrf_token
        );

        (url, csrf_token)
    }

    /// Exchange code for token
    pub async fn exchange_code(&self, code: &str) -> anyhow::Result<Credentials> {
        let params = [
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &self.config.redirect_uri),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Token exchange failed: {}", error_text);
        }

        let token_data: TokenResponse = response.json().await?;

        let expires_at = token_data
            .expires_in
            .map(|secs| chrono::Utc::now() + chrono::Duration::seconds(secs as i64));

        Ok(Credentials {
            provider: "oauth".to_string(), // Caller should override
            token: token_data.access_token,
            refresh_token: token_data.refresh_token,
            expires_at,
        })
    }

    /// Refresh the access token using a refresh token
    pub async fn refresh_token(&self, refresh_token: &str) -> anyhow::Result<Credentials> {
        let params = [
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
            ("client_id", &self.config.client_id),
            ("client_secret", &self.config.client_secret),
        ];

        let response = self
            .client
            .post(&self.config.token_url)
            .form(&params)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Token refresh failed: {}", error_text);
        }

        let token_data: TokenResponse = response.json().await?;

        let expires_at = token_data
            .expires_in
            .map(|secs| chrono::Utc::now() + chrono::Duration::seconds(secs as i64));

        Ok(Credentials {
            provider: "oauth".to_string(), // Caller should override
            token: token_data.access_token,
            // Some providers return a new refresh token, others don't
            refresh_token: token_data
                .refresh_token
                .or_else(|| Some(refresh_token.to_string())),
            expires_at,
        })
    }
}
