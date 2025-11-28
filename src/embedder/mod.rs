pub mod client;
pub mod batcher;
pub mod types;
pub mod model;

#[cfg(test)]
mod tests;

pub use client::{EmbeddingClient, EmbedError};
pub use batcher::Batcher;
pub use types::{EmbeddingRequest, EmbeddingResponse};
pub use model::EmbeddingModelInfo;
