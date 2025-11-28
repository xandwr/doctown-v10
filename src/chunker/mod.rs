mod splitter;

#[cfg(test)]
mod tests;

pub use splitter::{Chunk, ChunkMetadata, chunk_semantic_units};

/// Unique identifier for a chunk
pub type ChunkId = u32;

/// Maximum target tokens per chunk (configurable)
pub const DEFAULT_MAX_TOKENS: usize = 2000;

/// Minimum tokens before considering merging
pub const MIN_MERGE_THRESHOLD: usize = 200;
