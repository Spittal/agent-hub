mod embedding;
mod oauth;
mod server;

pub use embedding::*;
pub use oauth::*;
pub use server::*;

use std::collections::HashMap;
use std::sync::Mutex;

pub struct AppState {
    pub servers: Vec<ServerConfig>,
    pub connections: HashMap<String, ConnectionState>,
    /// IDs of AI tool integrations that MCP Manager is configured to manage.
    pub enabled_integrations: Vec<String>,
    pub embedding_config: EmbeddingConfig,
}

pub struct ConnectionState {
    pub tools: Vec<McpTool>,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            servers: Vec::new(),
            connections: HashMap::new(),
            enabled_integrations: Vec::new(),
            embedding_config: EmbeddingConfig::default(),
        }
    }
}

pub type SharedState = Mutex<AppState>;
