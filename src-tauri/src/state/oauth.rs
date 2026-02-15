use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthTokens {
    pub access_token: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refresh_token: Option<String>,
    /// Seconds until access_token expires (from server response).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub expires_in: Option<u64>,
    /// Unix timestamp (seconds) when these tokens were obtained.
    pub obtained_at: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthServerMetadata {
    pub issuer: String,
    pub authorization_endpoint: String,
    pub token_endpoint: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub registration_endpoint: Option<String>,
    #[serde(default)]
    pub scopes_supported: Vec<String>,
    #[serde(default)]
    pub code_challenge_methods_supported: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OAuthState {
    pub auth_server_metadata: AuthServerMetadata,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokens: Option<OAuthTokens>,
}

pub struct OAuthStore {
    entries: HashMap<String, OAuthState>,
}

impl OAuthStore {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get(&self, server_id: &str) -> Option<&OAuthState> {
        self.entries.get(server_id)
    }

    pub fn set(&mut self, server_id: String, state: OAuthState) {
        self.entries.insert(server_id, state);
    }

    pub fn remove(&mut self, server_id: &str) -> Option<OAuthState> {
        self.entries.remove(server_id)
    }

    pub fn entries_mut(&mut self) -> &mut HashMap<String, OAuthState> {
        &mut self.entries
    }
}

pub type SharedOAuthStore = tokio::sync::Mutex<OAuthStore>;
