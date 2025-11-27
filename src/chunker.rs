use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;
use uuid::Uuid;
use walkdir::WalkDir;

use crate::db::{CodeChunk, FileInfo};

/// Detect programming language from file extension
pub fn detect_language(path: &Path) -> Option<String> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .and_then(|ext| {
            Some(match ext.to_lowercase().as_str() {
                "rs" => "rust",
                "py" => "python",
                "js" => "javascript",
                "ts" => "typescript",
                "jsx" => "javascript",
                "tsx" => "typescript",
                "go" => "go",
                "java" => "java",
                "c" => "c",
                "cpp" | "cc" | "cxx" => "cpp",
                "h" | "hpp" => "cpp",
                "cs" => "csharp",
                "rb" => "ruby",
                "php" => "php",
                "swift" => "swift",
                "kt" | "kts" => "kotlin",
                "scala" => "scala",
                "md" => "markdown",
                "json" => "json",
                "yaml" | "yml" => "yaml",
                "toml" => "toml",
                "sh" | "bash" => "bash",
                "html" => "html",
                "css" => "css",
                "sql" => "sql",
                _ => return None,
            }
            .to_string())
        })
}

/// Check if a file should be ignored
pub fn should_ignore(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Ignore patterns
    let ignore_patterns = [
        "target/",
        "node_modules/",
        ".git/",
        ".svn/",
        ".hg/",
        "build/",
        "dist/",
        "out/",
        "__pycache__/",
        ".pytest_cache/",
        ".mypy_cache/",
        ".tox/",
        "venv/",
        "env/",
        ".env/",
        "vendor/",
        "Cargo.lock",
        "package-lock.json",
        "yarn.lock",
        "*.min.js",
        "*.min.css",
        "*.map",
    ];

    for pattern in &ignore_patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }

    false
}

/// Compute SHA256 hash of file contents
pub fn hash_file(path: &Path) -> Result<String> {
    let contents = fs::read(path)
        .context(format!("Failed to read file: {}", path.display()))?;
    let mut hasher = Sha256::new();
    hasher.update(&contents);
    Ok(hex::encode(hasher.finalize()))
}

/// Chunk a file into code segments
pub fn chunk_file(file_path: &str, content: &str, language: &str) -> Result<Vec<CodeChunk>> {
    let mut chunks = Vec::new();

    // Simple line-based chunking (can be enhanced with AST parsing later)
    let lines: Vec<&str> = content.lines().collect();
    
    if lines.is_empty() {
        return Ok(chunks);
    }

    // Configuration
    const CHUNK_SIZE: usize = 100; // lines per chunk
    const OVERLAP: usize = 10;     // overlap between chunks

    let mut start = 0;
    while start < lines.len() {
        let end = (start + CHUNK_SIZE).min(lines.len());
        let chunk_lines = &lines[start..end];
        let chunk_content = chunk_lines.join("\n");

        // Skip empty chunks
        if chunk_content.trim().is_empty() {
            start = end;
            continue;
        }

        let chunk_id = Uuid::new_v4().to_string();
        
        chunks.push(CodeChunk {
            id: chunk_id,
            file_path: file_path.to_string(),
            content: chunk_content,
            start_line: (start + 1) as u32, // 1-indexed
            end_line: end as u32,
            language: language.to_string(),
            chunk_type: "code_block".to_string(),
            name: None,
        });

        // Move to next chunk with overlap
        if end >= lines.len() {
            break;
        }
        start = end.saturating_sub(OVERLAP);
    }

    Ok(chunks)
}

/// Walk directory and collect file information
pub fn collect_files(repo_path: &str) -> Result<Vec<(FileInfo, String)>> {
    eprintln!("[chunker] Scanning repository: {}", repo_path);
    
    let mut files = Vec::new();
    let mut total_size = 0u64;
    let mut file_count = 0u32;

    for entry in WalkDir::new(repo_path)
        .follow_links(false)
        .into_iter()
        .filter_entry(|e| !should_ignore(e.path()))
    {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        // Only process files with recognized languages
        let language = match detect_language(path) {
            Some(lang) => lang,
            None => continue,
        };

        // Read file contents
        let content = match fs::read_to_string(path) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("[chunker] Warning: Failed to read {}: {}", path.display(), e);
                continue;
            }
        };

        // Get file info
        let metadata = fs::metadata(path)
            .context(format!("Failed to get metadata for {}", path.display()))?;
        let size = metadata.len();
        total_size += size;

        let hash = hash_file(path)
            .context(format!("Failed to hash file: {}", path.display()))?;

        let relative_path = path
            .strip_prefix(repo_path)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        let file_info = FileInfo {
            path: relative_path.clone(),
            hash,
            size,
            language: language.clone(),
        };

        files.push((file_info, content));
        file_count += 1;

        if file_count % 10 == 0 {
            eprintln!("[chunker] Processed {} files...", file_count);
        }
    }

    eprintln!("[chunker] ✓ Found {} files ({} bytes total)", file_count, total_size);
    Ok(files)
}

/// Process repository and generate chunks
pub fn process_repository(repo_path: &str) -> Result<(Vec<FileInfo>, Vec<CodeChunk>)> {
    eprintln!("[chunker] Processing repository...");
    
    let files_with_content = collect_files(repo_path)
        .context("Failed to collect files from repository")?;

    let mut all_file_infos = Vec::new();
    let mut all_chunks = Vec::new();
    let mut total_chunks = 0u32;

    for (file_info, content) in files_with_content {
        // Generate chunks for this file
        let chunks = chunk_file(&file_info.path, &content, &file_info.language)
            .context(format!("Failed to chunk file: {}", file_info.path))?;

        total_chunks += chunks.len() as u32;
        all_chunks.extend(chunks);
        all_file_infos.push(file_info);
    }

    eprintln!("[chunker] ✓ Generated {} chunks from {} files", 
              total_chunks, all_file_infos.len());

    Ok((all_file_infos, all_chunks))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_language() {
        assert_eq!(detect_language(Path::new("test.rs")), Some("rust".to_string()));
        assert_eq!(detect_language(Path::new("test.py")), Some("python".to_string()));
        assert_eq!(detect_language(Path::new("test.js")), Some("javascript".to_string()));
        assert_eq!(detect_language(Path::new("test.unknown")), None);
    }

    #[test]
    fn test_should_ignore() {
        assert!(should_ignore(Path::new("target/debug/foo")));
        assert!(should_ignore(Path::new("node_modules/package/index.js")));
        assert!(should_ignore(Path::new(".git/config")));
        assert!(!should_ignore(Path::new("src/main.rs")));
    }
}
