// the python contract
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct EmbeddingRequest {
    pub texts: Vec<String>,
}

#[derive(Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
}
