// Public API exports
pub mod chunker;
pub mod clusterer;
pub mod embedder;
pub mod parser;
pub mod sandbox;
pub mod security;

// Re-export main types for convenience
pub use sandbox::{FileEntry, Sandbox, SandboxBuilder, SandboxError};
pub use security::PathSanitizer;

pub use parser::{
    FileMetadata, ParseResult, Parser, ParserRegistry, SemanticKind, SemanticUnit, UnknownParser,
};

pub use chunker::{Chunk, ChunkId, ChunkMetadata, DEFAULT_MAX_TOKENS, chunk_semantic_units};

pub use embedder::{Batcher, EmbedError, EmbeddingClient, EmbeddingModelInfo};

pub use clusterer::{Cluster, ClusterResult, kmeans};
