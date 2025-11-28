pub mod batcher;
pub mod client;
pub mod model;
pub mod types;

#[cfg(test)]
mod tests;

pub use batcher::Batcher;
pub use client::{EmbedError, EmbeddingClient};
pub use model::EmbeddingModelInfo;
pub use types::{EmbeddingRequest, EmbeddingResponse};
