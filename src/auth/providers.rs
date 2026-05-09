use super::OAuthConfig;
use crate::plugins::traits::{AuthProvider, AuthType, Credentials};

pub struct GoogleProvider {
    pub config: OAuthConfig,
}

impl GoogleProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            config: OAuthConfig {
                client_id,
                client_secret,
                redirect_uri,
                scopes: vec![
                    "https://www.googleapis.com/auth/userinfo.email".into(),
                    "https://www.googleapis.com/auth/userinfo.profile".into(),
                    "https://www.googleapis.com/auth/cloud-platform".into(),
                ],
                auth_url: "https://accounts.google.com/o/oauth2/v2/auth".into(),
                token_url: "https://oauth2.googleapis.com/token".into(),
            },
        }
    }
}

impl AuthProvider for GoogleProvider {
    fn name(&self) -> &str {
        "google"
    }

    fn auth_type(&self) -> AuthType {
        AuthType::OAuth
    }

    fn authenticate(&self) -> anyhow::Result<Credentials> {
        // TODO: Trigger OAuth flow
        // Return dummy credentials for now to satisfy trait
        Ok(Credentials::default())
    }
}

pub struct GitHubProvider {
    pub config: OAuthConfig,
}

impl GitHubProvider {
    pub fn new(client_id: String, client_secret: String, redirect_uri: String) -> Self {
        Self {
            config: OAuthConfig {
                client_id,
                client_secret,
                redirect_uri,
                scopes: vec!["user", "repo", "read:org"]
                    .into_iter()
                    .map(String::from)
                    .collect(),
                auth_url: "https://github.com/login/oauth/authorize".into(),
                token_url: "https://github.com/login/oauth/access_token".into(),
            },
        }
    }
}

impl AuthProvider for GitHubProvider {
    fn name(&self) -> &str {
        "github"
    }

    fn auth_type(&self) -> AuthType {
        AuthType::OAuth
    }

    fn authenticate(&self) -> anyhow::Result<Credentials> {
        Ok(Credentials::default())
    }
}
