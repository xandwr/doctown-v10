// model.rs - stores metadata about the model you’re using
pub struct EmbeddingModelInfo {
    pub dim: usize,
    pub name: String,
    pub max_batch: usize,
}

// Doctown docpacks will eventually want to store:
// embedding dimension
// model name
// version