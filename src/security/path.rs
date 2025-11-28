use crate::sandbox::SandboxError;
use std::path::{Component, Path};

pub struct PathSanitizer;

impl PathSanitizer {
    /// Sanitize a path from a ZIP archive to prevent:
    /// - Directory traversal (../)
    /// - Absolute paths (/etc/passwd)
    /// - Zip slip attacks
    ///
    /// Hidden files (starting with .) are allowed for analysis purposes.
    /// Returns a normalized virtual path or an error.
    pub fn sanitize(raw_path: &str) -> Result<String, SandboxError> {
        Self::sanitize_with_options(raw_path, true)
    }

    /// Sanitize with custom options
    pub fn sanitize_with_options(
        raw_path: &str,
        allow_hidden: bool,
    ) -> Result<String, SandboxError> {
        // Reject empty paths
        if raw_path.is_empty() {
            return Err(SandboxError::InvalidPath("Empty path".to_string()));
        }

        let path = Path::new(raw_path);
        let mut components = Vec::new();

        for component in path.components() {
            match component {
                // Reject absolute paths
                Component::Prefix(_) | Component::RootDir => {
                    return Err(SandboxError::InvalidPath(format!(
                        "Absolute path not allowed: {}",
                        raw_path
                    )));
                }
                // Reject parent directory traversal
                Component::ParentDir => {
                    return Err(SandboxError::InvalidPath(format!(
                        "Parent directory traversal not allowed: {}",
                        raw_path
                    )));
                }
                // Skip current directory markers
                Component::CurDir => continue,
                // Accept normal components
                Component::Normal(part) => {
                    let part_str = part.to_str().ok_or_else(|| {
                        SandboxError::InvalidPath(format!("Invalid UTF-8 in path: {:?}", part))
                    })?;

                    // Optionally reject hidden files/directories
                    if !allow_hidden && part_str.starts_with('.') {
                        return Err(SandboxError::InvalidPath(format!(
                            "Hidden files not allowed: {}",
                            raw_path
                        )));
                    }

                    components.push(part_str);
                }
            }
        }

        // Reject if no valid components remain
        if components.is_empty() {
            return Err(SandboxError::InvalidPath(format!(
                "No valid components: {}",
                raw_path
            )));
        }

        // Build normalized path with forward slashes
        Ok(components.join("/"))
    }
}
