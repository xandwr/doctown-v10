use anyhow::{Context, Result};
use chrono::{Datelike, Timelike, Utc};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{Read, Write};
use zip::write::FileOptions;
use zip::{CompressionMethod, ZipArchive, ZipWriter};

use crate::db::DocpackDB;

#[derive(Debug, Serialize, Deserialize)]
pub struct Manifest {
    pub version: String,
    pub created_at: String,
    pub source_repo: Option<String>,
    pub source_path: Option<String>,
    pub generator: String,
    pub stats: ManifestStats,
    pub models: ModelInfo,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ManifestStats {
    pub file_count: u32,
    pub chunk_count: u32,
    pub embedding_count: u32,
    pub symbol_count: u32,
    pub total_size_bytes: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ModelInfo {
    pub embedding_model: String,
    pub reranker_model: Option<String>,
    pub generator_model: Option<String>,
}

pub struct DocpackWriter {
    db: DocpackDB,
    manifest: Manifest,
}

impl DocpackWriter {
    /// Create a new docpack writer
    pub fn new(
        source_repo: Option<String>,
        source_path: Option<String>,
        embedding_model: String,
    ) -> Result<Self> {
        let db = DocpackDB::new_in_memory()
            .context("Failed to create in-memory database")?;

        let manifest = Manifest {
            version: "1.0.0".to_string(),
            created_at: Utc::now().to_rfc3339(),
            source_repo,
            source_path,
            generator: format!("doctown v{}", env!("CARGO_PKG_VERSION")),
            stats: ManifestStats {
                file_count: 0,
                chunk_count: 0,
                embedding_count: 0,
                symbol_count: 0,
                total_size_bytes: 0,
            },
            models: ModelInfo {
                embedding_model,
                reranker_model: None,
                generator_model: None,
            },
        };

        Ok(Self { db, manifest })
    }

    /// Get mutable reference to the database
    pub fn db_mut(&mut self) -> &mut DocpackDB {
        &mut self.db
    }

    /// Get reference to the database
    pub fn db(&self) -> &DocpackDB {
        &self.db
    }

    /// Update manifest stats from database
    fn update_stats(&mut self) -> Result<()> {
        let stats = self.db.get_stats()
            .context("Failed to get database stats")?;
        
        self.manifest.stats.file_count = stats.file_count;
        self.manifest.stats.chunk_count = stats.chunk_count;
        self.manifest.stats.embedding_count = stats.embedding_count;
        self.manifest.stats.symbol_count = stats.symbol_count;
        
        Ok(())
    }

    /// Set optional models in manifest
    pub fn set_reranker_model(&mut self, model: String) {
        self.manifest.models.reranker_model = Some(model);
    }

    pub fn set_generator_model(&mut self, model: String) {
        self.manifest.models.generator_model = Some(model);
    }

    /// Write docpack to file
    pub fn write_to_file(&mut self, output_path: &str) -> Result<()> {
        eprintln!("[docpack] Writing docpack to: {}", output_path);
        
        // Update stats before writing
        self.update_stats()
            .context("Failed to update manifest stats")?;

        // Create temporary database file
        let temp_db_path = format!("{}.tmp.db", output_path);
        self.db.save_to_file(&temp_db_path)
            .context("Failed to save database to temporary file")?;

        // Create ZIP file
        let file = File::create(output_path)
            .context(format!("Failed to create output file: {}", output_path))?;
        let mut zip = ZipWriter::new(file);
        // Set proper file options with current timestamp
        let now = chrono::Local::now();
        let options: FileOptions<'_, ()> = FileOptions::default()
            .compression_method(CompressionMethod::Deflated)
            .unix_permissions(0o666)
            .last_modified_time(
                zip::DateTime::from_date_and_time(
                    now.year() as u16,
                    now.month() as u8,
                    now.day() as u8,
                    now.hour() as u8,
                    now.minute() as u8,
                    now.second() as u8,
                ).unwrap_or_default()
            );

        // Add database
        eprintln!("[docpack] Adding docpack.sqlite to archive...");
        zip.start_file("docpack.sqlite", options)
            .context("Failed to start database file in ZIP")?;
        let mut db_file = File::open(&temp_db_path)
            .context("Failed to open temporary database file")?;
        let mut db_contents = Vec::new();
        db_file.read_to_end(&mut db_contents)
            .context("Failed to read database contents")?;
        zip.write_all(&db_contents)
            .context("Failed to write database to ZIP")?;

        // Add manifest.json
        eprintln!("[docpack] Adding manifest.json to archive...");
        zip.start_file("manifest.json", options)
            .context("Failed to start manifest file in ZIP")?;
        let manifest_json = serde_json::to_string_pretty(&self.manifest)
            .context("Failed to serialize manifest")?;
        zip.write_all(manifest_json.as_bytes())
            .context("Failed to write manifest to ZIP")?;

        // Add readme.md
        eprintln!("[docpack] Adding readme.md to archive...");
        zip.start_file("readme.md", options)
            .context("Failed to start readme file in ZIP")?;
        let readme = self.generate_readme();
        zip.write_all(readme.as_bytes())
            .context("Failed to write readme to ZIP")?;

        // Create assets directory (empty for now)
        zip.add_directory("assets/", options)
            .context("Failed to create assets directory")?;

        // Finish ZIP
        zip.finish()
            .context("Failed to finalize ZIP file")?;

        // Set proper file permissions (readable and writable by owner and group)
        #[cfg(unix)]
        {
            use std::fs;
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o666);
            fs::set_permissions(output_path, permissions)
                .context("Failed to set file permissions")?;
        }

        // Clean up temporary database file
        std::fs::remove_file(&temp_db_path)
            .context("Failed to remove temporary database file")?;

        eprintln!("[docpack] ✓ Successfully created docpack: {}", output_path);
        eprintln!("[docpack]   Files: {}", self.manifest.stats.file_count);
        eprintln!("[docpack]   Chunks: {}", self.manifest.stats.chunk_count);
        eprintln!("[docpack]   Embeddings: {}", self.manifest.stats.embedding_count);
        eprintln!("[docpack]   Symbols: {}", self.manifest.stats.symbol_count);

        Ok(())
    }

    /// Generate a human-readable README
    fn generate_readme(&self) -> String {
        format!(
            r#"# Docpack

This is a docpack generated by doctown.

## Metadata

- **Version**: {}
- **Created**: {}
- **Generator**: {}
- **Source**: {}

## Contents

- **Files**: {}
- **Code Chunks**: {}
- **Embeddings**: {}
- **Symbols**: {}

## Models Used

- **Embedding**: {}
- **Reranker**: {}
- **Generator**: {}

## Structure

```
docpack.sqlite    - All structured data (chunks, embeddings, symbols)
manifest.json     - Top-level metadata
assets/           - Screenshots, diagrams, attachments
readme.md         - This file
```

## Usage

This docpack can be queried using the doctown tool or any SQLite-compatible database viewer.

"#,
            self.manifest.version,
            self.manifest.created_at,
            self.manifest.generator,
            self.manifest.source_repo.as_ref()
                .or(self.manifest.source_path.as_ref())
                .unwrap_or(&"Unknown".to_string()),
            self.manifest.stats.file_count,
            self.manifest.stats.chunk_count,
            self.manifest.stats.embedding_count,
            self.manifest.stats.symbol_count,
            self.manifest.models.embedding_model,
            self.manifest.models.reranker_model.as_ref().unwrap_or(&"None".to_string()),
            self.manifest.models.generator_model.as_ref().unwrap_or(&"None".to_string()),
        )
    }
}

pub struct DocpackReader {
    db: DocpackDB,
    pub manifest: Manifest,
}

impl DocpackReader {
    /// Open and read a docpack file
    pub fn open(docpack_path: &str) -> Result<Self> {
        eprintln!("[docpack] Opening docpack: {}", docpack_path);

        let file = File::open(docpack_path)
            .context(format!("Failed to open docpack: {}", docpack_path))?;
        let mut archive = ZipArchive::new(file)
            .context("Failed to read ZIP archive")?;

        // Extract manifest
        let mut manifest_file = archive.by_name("manifest.json")
            .context("manifest.json not found in docpack")?;
        let mut manifest_contents = String::new();
        manifest_file.read_to_string(&mut manifest_contents)
            .context("Failed to read manifest")?;
        let manifest: Manifest = serde_json::from_str(&manifest_contents)
            .context("Failed to parse manifest.json")?;
        // Drop the `manifest_file` before borrowing `archive` mutably again.
        drop(manifest_file);

        eprintln!("[docpack] Manifest loaded: {} chunks, {} embeddings", 
                  manifest.stats.chunk_count, manifest.stats.embedding_count);

        // Extract database to temporary file
        let temp_db_path = format!("{}.extracted.db", docpack_path);
        let mut db_file = archive.by_name("docpack.sqlite")
            .context("docpack.sqlite not found in archive")?;
        let mut db_contents = Vec::new();
        db_file.read_to_end(&mut db_contents)
            .context("Failed to read database from archive")?;
        
        let mut temp_file = File::create(&temp_db_path)
            .context("Failed to create temporary database file")?;
        temp_file.write_all(&db_contents)
            .context("Failed to write temporary database file")?;

        // Open database
        let db = DocpackDB::open(&temp_db_path)
            .context("Failed to open extracted database")?;

        eprintln!("[docpack] ✓ Docpack loaded successfully");

        Ok(Self { db, manifest })
    }

    /// Get reference to the database
    pub fn db(&self) -> &DocpackDB {
        &self.db
    }
}
