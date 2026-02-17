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
