mod registry;
mod result;
mod unknown;

pub use registry::ParserRegistry;
pub use result::{FileMetadata, ParseResult, SemanticKind, SemanticUnit};
pub use unknown::UnknownParser;

/// Core trait that all parsers must implement
pub trait Parser: Send + Sync {
    /// Parse raw bytes into a normalized representation
    ///
    /// # Arguments
    /// * `path` - Virtual path from sandbox (e.g., "src/main.rs")
    /// * `bytes` - Raw file contents from arena
    ///
    /// # Returns
    /// Normalized text, metadata, and semantic units for chunking
    fn parse(&self, path: &str, bytes: &[u8]) -> ParseResult;
}
