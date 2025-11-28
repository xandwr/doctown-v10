// model.rs - stores metadata about the model you're using
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingModelInfo {
    pub dim: usize,
    pub name: String,
    pub max_batch: usize,
}

impl EmbeddingModelInfo {
    pub fn new(name: impl Into<String>, dim: usize, max_batch: usize) -> Self {
        Self {
            name: name.into(),
            dim,
            max_batch,
        }
    }

    pub fn gemma_300m() -> Self {
        Self::new("google/embeddinggemma-300m", 768, 32)
    }
}

impl Default for EmbeddingModelInfo {
    fn default() -> Self {
        Self::gemma_300m()
    }
}
