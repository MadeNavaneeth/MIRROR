use crate::auth::oauth::OAuthFlow;
use crate::auth::providers::{GitHubProvider, GoogleProvider};
use crate::auth::storage::CredentialStorage;
use crate::config::Config;
use anyhow::Result;
use axum::{extract::Query, response::Html, routing::get, Router};
use serde::Deserialize;
use std::sync::{Arc, Mutex};
use tokio::net::TcpListener;
use tokio::sync::oneshot;

#[derive(Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

pub struct AuthHandler {
    config: Config,
    storage: Arc<CredentialStorage>,
}

impl AuthHandler {
    pub fn new(config: Config, storage: Arc<CredentialStorage>) -> Self {
        Self { config, storage }
    }

    pub async fn login(&self, provider_name: &str) -> Result<()> {
        match provider_name {
            "google" | "github" => self.login_oauth(provider_name).await,
            _ => anyhow::bail!("Unsupported provider: {}", provider_name),
        }
    }

    async fn login_oauth(&self, provider_name: &str) -> Result<()> {
        let oauth_config = match provider_name {
            "google" => {
                let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
                let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
                GoogleProvider::new(
                    client_id,
                    client_secret,
                    "http://localhost:8080/callback".to_string(),
                )
                .config
            }
            "github" => {
                let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
                let client_secret = std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();
                GitHubProvider::new(
                    client_id,
                    client_secret,
                    "http://localhost:8080/callback".to_string(),
                )
                .config
            }
            _ => anyhow::bail!("Unsupported provider: {}", provider_name),
        };

        let flow = OAuthFlow::new(oauth_config);
        let (auth_url, _csrf_token) = flow.get_auth_url();

        println!("🔗 Opening browser for {} login...", provider_name);
        println!("Authorization URL: {}", auth_url);
        let _ = open::that(&auth_url);

        let (tx, rx) = oneshot::channel::<String>();
        let tx_mutex = Arc::new(Mutex::new(Some(tx)));
        let provider_name_for_app = provider_name.to_string();

        let app = Router::new()
            .route("/callback", get(move |Query(params): Query<CallbackQuery>| {
                let tx_mutex = tx_mutex.clone();
                let provider_name = provider_name_for_app.clone();
                async move {
                    if let Some(tx) = tx_mutex.lock().unwrap().take() {
                        let _ = tx.send(params.code);
                    }
                    Html(format!("<h1>{} Login Successful!</h1><p>You can close this window and return to the terminal.</p>", provider_name))
                }
            }));

        let listener = TcpListener::bind("127.0.0.1:8080").await?;
        println!("📡 Listening for callback on http://localhost:8080/callback...");

        let server = axum::serve(listener, app);

        tokio::select! {
            _ = server => {
                anyhow::bail!("Server stopped unexpectedly");
            }
            code = rx => {
                let code = code?;
                println!("✅ Received authorization code. Exchanging for tokens...");
                let mut credentials = flow.exchange_code(&code).await?;
                credentials.provider = provider_name.to_string();

                self.storage.save(provider_name, credentials)?;
                println!("✨ Successfully logged in to {} and saved credentials.", provider_name);
            }
        }

        Ok(())
    }

    pub fn list(&self) -> Result<()> {
        let providers = self.storage.list_providers();
        if providers.is_empty() {
            println!("📭 No authenticated providers found.");
        } else {
            println!("🔐 Authenticated providers:");
            for p in providers {
                println!("  - {}", p);
            }
        }
        Ok(())
    }

    pub async fn logout(&self, provider: &str) -> Result<()> {
        self.storage.delete(provider)?;
        println!("👋 Logged out from {}.", provider);
        Ok(())
    }

    pub async fn ensure_google_token(&self) -> Result<String> {
        self.ensure_token_valid("google").await
    }

    pub async fn ensure_token_valid(&self, provider_name: &str) -> Result<String> {
        let creds = self.storage.get(provider_name).ok_or_else(|| {
            anyhow::anyhow!(
                "Not logged in to {}. Run 'mirror auth login {}' first.",
                provider_name,
                provider_name
            )
        })?;

        let now = chrono::Utc::now();
        let buffer = chrono::Duration::minutes(5);

        if let Some(expires_at) = creds.expires_at {
            if expires_at - buffer > now {
                return Ok(creds.token);
            }
        }

        let refresh_token = creds.refresh_token.as_ref().ok_or_else(|| {
            anyhow::anyhow!("Token expired and no refresh token available. Re-login required.")
        })?;

        tracing::info!("🔄 {} token expired, refreshing...", provider_name);

        let oauth_config = match provider_name {
            "google" => {
                let client_id = std::env::var("GOOGLE_CLIENT_ID").unwrap_or_default();
                let client_secret = std::env::var("GOOGLE_CLIENT_SECRET").unwrap_or_default();
                GoogleProvider::new(
                    client_id,
                    client_secret,
                    "http://localhost:8080/callback".to_string(),
                )
                .config
            }
            "github" => {
                let client_id = std::env::var("GITHUB_CLIENT_ID").unwrap_or_default();
                let client_secret = std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default();
                GitHubProvider::new(
                    client_id,
                    client_secret,
                    "http://localhost:8080/callback".to_string(),
                )
                .config
            }
            _ => anyhow::bail!("Unsupported provider: {}", provider_name),
        };

        let flow = OAuthFlow::new(oauth_config);
        let mut new_creds = flow.refresh_token(refresh_token).await?;
        new_creds.provider = provider_name.to_string();

        let token = new_creds.token.clone();
        self.storage.save(provider_name, new_creds)?;

        Ok(token)
    }
}
