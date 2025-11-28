/// Metadata for a single file in the sandbox arena
#[derive(Debug, Clone)]
pub struct FileEntry {
    /// Byte offset into the arena
    pub offset: usize,
    /// Length in bytes
    pub length: usize,
    /// Sanitized virtual path (e.g., "src/lib.rs")
    pub virtual_path: String,
}
