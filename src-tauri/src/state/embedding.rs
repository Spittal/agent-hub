use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct EmbeddingConfig {
    pub provider: EmbeddingProvider,
    pub model: String,
    pub dimensions: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum EmbeddingProvider {
    Ollama,
    Openai,
}

impl Default for EmbeddingConfig {
    fn default() -> Self {
        Self {
            provider: EmbeddingProvider::Openai,
            model: "text-embedding-3-small".into(),
            dimensions: 1536,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct RedisConfig {
    pub source: RedisSource,
    pub url: Option<String>,
    /// Shell command to run before connecting (e.g. gcloud SSH tunnel). Only used when source == Remote.
    pub tunnel_command: Option<String>,
    /// Local port the tunnel binds to. Defaults to 6379 if not set.
    pub tunnel_local_port: Option<u16>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RedisSource {
    Local,
    Remote,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            source: RedisSource::Local,
            url: None,
            tunnel_command: None,
            tunnel_local_port: None,
        }
    }
}
