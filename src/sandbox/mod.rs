mod entry;
mod error;

pub use entry::FileEntry;
pub use error::SandboxError;

use crate::security::PathSanitizer;
use std::collections::HashMap;
use std::io::{Cursor, Read};

/// Immutable sandbox with arena-backed file storage
pub struct Sandbox {
    /// Single contiguous blob containing all file data
    arena: Vec<u8>,
    /// Index mapping virtual paths to arena slices
    index: HashMap<String, FileEntry>,
}

/// Mutable builder for constructing a sandbox
pub struct SandboxBuilder {
    arena: Vec<u8>,
    index: HashMap<String, FileEntry>,
    max_file_size: u64,
    max_total_size: u64,
}

impl SandboxBuilder {
    /// Create a new builder with default limits
    pub fn new() -> Self {
        Self {
            arena: Vec::new(),
            index: HashMap::new(),
            max_file_size: 50 * 1024 * 1024,   // 50 MB per file
            max_total_size: 500 * 1024 * 1024, // 500 MB total
        }
    }

    /// Set maximum individual file size
    pub fn max_file_size(mut self, size: u64) -> Self {
        self.max_file_size = size;
        self
    }

    /// Set maximum total arena size
    pub fn max_total_size(mut self, size: u64) -> Self {
        self.max_total_size = size;
        self
    }

    /// Add a file to the sandbox arena
    pub fn add_file(&mut self, raw_path: &str, data: &[u8]) -> Result<(), SandboxError> {
        // Sanitize the path
        let virtual_path = PathSanitizer::sanitize(raw_path)?;

        // Check file size limit
        if data.len() as u64 > self.max_file_size {
            return Err(SandboxError::FileTooLarge {
                size: data.len() as u64,
                max: self.max_file_size,
            });
        }

        // Check total size limit
        let new_total = self.arena.len() as u64 + data.len() as u64;
        if new_total > self.max_total_size {
            return Err(SandboxError::FileTooLarge {
                size: new_total,
                max: self.max_total_size,
            });
        }

        // Add to arena
        let offset = self.arena.len();
        self.arena.extend_from_slice(data);
        let length = data.len();

        // Add to index
        self.index.insert(
            virtual_path.clone(),
            FileEntry {
                offset,
                length,
                virtual_path,
            },
        );

        Ok(())
    }

    /// Ingest a GitHub repository as a ZIP archive
    pub fn ingest_github_repo(
        mut self,
        owner: &str,
        repo: &str,
        branch: &str,
    ) -> Result<Self, SandboxError> {
        // Construct GitHub ZIP URL
        let url = format!(
            "https://github.com/{}/{}/archive/refs/heads/{}.zip",
            owner, repo, branch
        );

        // Download ZIP
        let response = reqwest::blocking::get(&url)
            .map_err(|e| SandboxError::DownloadFailed(format!("HTTP request failed: {}", e)))?;

        if !response.status().is_success() {
            return Err(SandboxError::DownloadFailed(format!(
                "HTTP {}: {}",
                response.status(),
                response.status().canonical_reason().unwrap_or("Unknown")
            )));
        }

        let bytes = response.bytes().map_err(|e| {
            SandboxError::DownloadFailed(format!("Failed to read response body: {}", e))
        })?;

        // Parse ZIP in memory
        let cursor = Cursor::new(bytes);
        let mut archive = zip::ZipArchive::new(cursor)
            .map_err(|e| SandboxError::ZipParseFailed(e.to_string()))?;

        // Extract all files into the arena
        for i in 0..archive.len() {
            let mut file = archive
                .by_index(i)
                .map_err(|e| SandboxError::ZipParseFailed(e.to_string()))?;

            // Skip directories
            if file.is_dir() {
                continue;
            }

            // Get the file path from the ZIP
            let raw_path = file.name().to_string();

            // GitHub ZIPs have a top-level directory like "repo-main/"
            // Strip it to get clean paths
            let stripped_path = raw_path
                .split_once('/')
                .map(|(_, rest)| rest)
                .unwrap_or(&raw_path);

            // Skip if empty after stripping
            if stripped_path.is_empty() {
                continue;
            }

            // Read file contents
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .map_err(|e| SandboxError::ZipParseFailed(e.to_string()))?;

            // Add to sandbox (this handles sanitization)
            self.add_file(stripped_path, &contents)?;
        }

        Ok(self)
    }

    /// Build the immutable sandbox
    pub fn build(self) -> Sandbox {
        Sandbox {
            arena: self.arena,
            index: self.index,
        }
    }
}

impl Default for SandboxBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Sandbox {
    /// Get a file's contents as a byte slice (zero-copy)
    pub fn get(&self, virtual_path: &str) -> Option<&[u8]> {
        self.index
            .get(virtual_path)
            .map(|entry| &self.arena[entry.offset..entry.offset + entry.length])
    }

    /// List all files in the sandbox
    pub fn list(&self) -> impl Iterator<Item = &FileEntry> {
        self.index.values()
    }

    /// Walk all files under a given directory prefix
    pub fn walk_prefix(&self, dir_prefix: &str) -> Vec<&FileEntry> {
        let normalized_prefix = if dir_prefix.is_empty() {
            String::new()
        } else {
            format!("{}/", dir_prefix.trim_end_matches('/'))
        };

        self.index
            .values()
            .filter(|entry| {
                if normalized_prefix.is_empty() {
                    true // Match all if prefix is empty
                } else {
                    entry.virtual_path.starts_with(&normalized_prefix)
                }
            })
            .collect()
    }

    /// Get metadata for a file without reading contents
    pub fn get_entry(&self, virtual_path: &str) -> Option<&FileEntry> {
        self.index.get(virtual_path)
    }

    /// Get the total number of files
    pub fn file_count(&self) -> usize {
        self.index.len()
    }

    /// Get the total arena size in bytes
    pub fn total_size(&self) -> usize {
        self.arena.len()
    }
}
