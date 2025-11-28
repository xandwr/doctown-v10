// the python contract
use serde::{Serialize, Deserialize};

#[derive(Serialize)]
pub struct EmbeddingRequest {
    pub texts: Vec<String>,
}

#[derive(Deserialize)]
pub struct EmbeddingResponse {
    pub embeddings: Vec<Vec<f32>>,
}
