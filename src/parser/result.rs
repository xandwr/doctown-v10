/// Result of parsing any file type
#[derive(Debug, Clone)]
pub struct ParseResult {
    /// Normalized UTF-8 text representation
    pub normalized_text: String,
    /// File metadata and heuristics
    pub metadata: FileMetadata,
    /// Semantic units for chunk splitting
    pub semantic_units: Vec<SemanticUnit>,
}

/// Metadata extracted during parsing
#[derive(Debug, Clone)]
pub struct FileMetadata {
    /// Virtual path from sandbox
    pub path: String,
    /// File extension (e.g., "rs", "py", "md")
    pub extension: String,
    /// Detected language/type
    pub language: String,
    /// Original byte size
    pub size_bytes: usize,
    /// Line count in normalized text
    pub line_count: usize,
    /// Whether file is valid UTF-8
    pub is_utf8: bool,
}

/// A semantic unit representing a chunkable section
#[derive(Debug, Clone)]
pub struct SemanticUnit {
    /// Text content of this unit
    pub text: String,
    /// Byte offset in original file (start)
    pub start_offset: usize,
    /// Byte offset in original file (end)
    pub end_offset: usize,
    /// Semantic type of this unit
    pub kind: SemanticKind,
}

/// Classification of semantic units
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemanticKind {
    /// Unknown or unclassified content
    Unknown,
    /// Generic blob (binary, unparseable, etc.)
    Blob,
    /// Function definition
    Function,
    /// Class/struct/type definition
    Class,
    /// Module/namespace
    Module,
    /// Comment block
    Comment,
    /// Markdown paragraph
    Paragraph,
    /// Markdown heading section
    Section,
    /// JSON object
    Object,
    /// Configuration block
    Config,
}

impl FileMetadata {
    /// Create metadata from a path and byte slice
    pub fn from_path_and_bytes(path: &str, bytes: &[u8]) -> Self {
        let extension = std::path::Path::new(path)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Try to detect if it's valid UTF-8
        let is_utf8 = std::str::from_utf8(bytes).is_ok();

        Self {
            path: path.to_string(),
            extension: extension.clone(),
            language: Self::guess_language(&extension),
            size_bytes: bytes.len(),
            line_count: 0, // Will be set after parsing
            is_utf8,
        }
    }

    /// Heuristic language detection from extension
    fn guess_language(ext: &str) -> String {
        match ext {
            "rs" => "rust",
            "py" => "python",
            "js" | "jsx" => "javascript",
            "ts" | "tsx" => "typescript",
            "go" => "go",
            "c" | "h" => "c",
            "cpp" | "cc" | "cxx" | "hpp" => "cpp",
            "java" => "java",
            "rb" => "ruby",
            "php" => "php",
            "cs" => "csharp",
            "swift" => "swift",
            "kt" => "kotlin",
            "md" => "markdown",
            "json" => "json",
            "yaml" | "yml" => "yaml",
            "toml" => "toml",
            "xml" => "xml",
            "html" => "html",
            "css" => "css",
            "sh" | "bash" => "shell",
            "sql" => "sql",
            _ => "unknown",
        }
        .to_string()
    }

    /// Update line count after text normalization
    pub fn set_line_count(&mut self, count: usize) {
        self.line_count = count;
    }
}
