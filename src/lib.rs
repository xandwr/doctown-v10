// Public API exports
pub mod parser;
pub mod sandbox;
pub mod security;

// Re-export main types for convenience
pub use sandbox::{FileEntry, Sandbox, SandboxBuilder, SandboxError};
pub use security::PathSanitizer;

pub use parser::{
    FileMetadata, ParseResult, Parser, ParserRegistry, SemanticKind, SemanticUnit, UnknownParser,
};
