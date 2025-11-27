use anyhow::{Context, Result};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeChunk {
    pub id: String,
    pub file_path: String,
    pub content: String,
    pub start_line: u32,
    pub end_line: u32,
    pub language: String,
    pub chunk_type: String, // "function", "class", "module", etc.
    pub name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Embedding {
    pub chunk_id: String,
    pub vector: Vec<f32>,
    pub model: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Symbol {
    pub id: String,
    pub name: String,
    pub kind: String, // "function", "class", "struct", "trait", etc.
    pub file_path: String,
    pub line: u32,
    pub signature: Option<String>,
    pub documentation: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    pub path: String,
    pub hash: String,
    pub size: u64,
    pub language: String,
}

pub struct DocpackDB {
    conn: Connection,
}

impl DocpackDB {
    /// Create a new in-memory database
    pub fn new_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .context("Failed to create in-memory database")?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Open an existing database file
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)
            .context(format!("Failed to open database at {}", path))?;
        let db = Self { conn };
        db.init_schema()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS files (
                path TEXT PRIMARY KEY,
                hash TEXT NOT NULL,
                size INTEGER NOT NULL,
                language TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS chunks (
                id TEXT PRIMARY KEY,
                file_path TEXT NOT NULL,
                content TEXT NOT NULL,
                start_line INTEGER NOT NULL,
                end_line INTEGER NOT NULL,
                language TEXT NOT NULL,
                chunk_type TEXT NOT NULL,
                name TEXT,
                FOREIGN KEY (file_path) REFERENCES files(path)
            );

            CREATE TABLE IF NOT EXISTS embeddings (
                chunk_id TEXT PRIMARY KEY,
                vector BLOB NOT NULL,
                model TEXT NOT NULL,
                FOREIGN KEY (chunk_id) REFERENCES chunks(id)
            );

            CREATE TABLE IF NOT EXISTS symbols (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                kind TEXT NOT NULL,
                file_path TEXT NOT NULL,
                line INTEGER NOT NULL,
                signature TEXT,
                documentation TEXT,
                FOREIGN KEY (file_path) REFERENCES files(path)
            );

            CREATE INDEX IF NOT EXISTS idx_chunks_file ON chunks(file_path);
            CREATE INDEX IF NOT EXISTS idx_symbols_file ON symbols(file_path);
            CREATE INDEX IF NOT EXISTS idx_symbols_name ON symbols(name);
            "#,
        )
        .context("Failed to initialize database schema")?;
        Ok(())
    }

    /// Insert a file record
    pub fn insert_file(&self, file: &FileInfo) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO files (path, hash, size, language) VALUES (?1, ?2, ?3, ?4)",
                params![file.path, file.hash, file.size, file.language],
            )
            .context(format!("Failed to insert file: {}", file.path))?;
        Ok(())
    }

    /// Insert a code chunk
    pub fn insert_chunk(&self, chunk: &CodeChunk) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO chunks (id, file_path, content, start_line, end_line, language, chunk_type, name) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    chunk.id,
                    chunk.file_path,
                    chunk.content,
                    chunk.start_line,
                    chunk.end_line,
                    chunk.language,
                    chunk.chunk_type,
                    chunk.name
                ],
            )
            .context(format!("Failed to insert chunk: {}", chunk.id))?;
        Ok(())
    }

    /// Insert an embedding
    pub fn insert_embedding(&self, embedding: &Embedding) -> Result<()> {
        // Convert Vec<f32> to bytes
        let vector_bytes: Vec<u8> = embedding
            .vector
            .iter()
            .flat_map(|f| f.to_le_bytes())
            .collect();

        self.conn
            .execute(
                "INSERT OR REPLACE INTO embeddings (chunk_id, vector, model) VALUES (?1, ?2, ?3)",
                params![embedding.chunk_id, vector_bytes, embedding.model],
            )
            .context(format!("Failed to insert embedding for chunk: {}", embedding.chunk_id))?;
        Ok(())
    }

    /// Insert a symbol
    pub fn insert_symbol(&self, symbol: &Symbol) -> Result<()> {
        self.conn
            .execute(
                "INSERT OR REPLACE INTO symbols (id, name, kind, file_path, line, signature, documentation) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                params![
                    symbol.id,
                    symbol.name,
                    symbol.kind,
                    symbol.file_path,
                    symbol.line,
                    symbol.signature,
                    symbol.documentation
                ],
            )
            .context(format!("Failed to insert symbol: {}", symbol.name))?;
        Ok(())
    }

    /// Get all chunks
    pub fn get_all_chunks(&self) -> Result<Vec<CodeChunk>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, file_path, content, start_line, end_line, language, chunk_type, name FROM chunks")
            .context("Failed to prepare statement")?;

        let chunks = stmt
            .query_map([], |row| {
                Ok(CodeChunk {
                    id: row.get(0)?,
                    file_path: row.get(1)?,
                    content: row.get(2)?,
                    start_line: row.get(3)?,
                    end_line: row.get(4)?,
                    language: row.get(5)?,
                    chunk_type: row.get(6)?,
                    name: row.get(7)?,
                })
            })
            .context("Failed to query chunks")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect chunks")?;

        Ok(chunks)
    }

    /// Get embedding for a chunk
    pub fn get_embedding(&self, chunk_id: &str) -> Result<Option<Embedding>> {
        let mut stmt = self
            .conn
            .prepare("SELECT chunk_id, vector, model FROM embeddings WHERE chunk_id = ?1")
            .context("Failed to prepare statement")?;

        let mut rows = stmt
            .query(params![chunk_id])
            .context("Failed to query embedding")?;

        if let Some(row) = rows.next().context("Failed to get next row")? {
            let chunk_id: String = row.get(0)?;
            let vector_bytes: Vec<u8> = row.get(1)?;
            let model: String = row.get(2)?;

            // Convert bytes back to Vec<f32>
            let vector: Vec<f32> = vector_bytes
                .chunks_exact(4)
                .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
                .collect();

            Ok(Some(Embedding {
                chunk_id,
                vector,
                model,
            }))
        } else {
            Ok(None)
        }
    }

    /// Get all symbols
    pub fn get_all_symbols(&self) -> Result<Vec<Symbol>> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name, kind, file_path, line, signature, documentation FROM symbols")
            .context("Failed to prepare statement")?;

        let symbols = stmt
            .query_map([], |row| {
                Ok(Symbol {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    file_path: row.get(3)?,
                    line: row.get(4)?,
                    signature: row.get(5)?,
                    documentation: row.get(6)?,
                })
            })
            .context("Failed to query symbols")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect symbols")?;

        Ok(symbols)
    }

    /// Get statistics
    pub fn get_stats(&self) -> Result<DocpackStats> {
        let file_count: u32 = self
            .conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .context("Failed to count files")?;

        let chunk_count: u32 = self
            .conn
            .query_row("SELECT COUNT(*) FROM chunks", [], |row| row.get(0))
            .context("Failed to count chunks")?;

        let embedding_count: u32 = self
            .conn
            .query_row("SELECT COUNT(*) FROM embeddings", [], |row| row.get(0))
            .context("Failed to count embeddings")?;

        let symbol_count: u32 = self
            .conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
            .context("Failed to count symbols")?;

        Ok(DocpackStats {
            file_count,
            chunk_count,
            embedding_count,
            symbol_count,
        })
    }

    /// Save database to file
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        // Use SQLite's VACUUM INTO command to persist an in-memory DB to a file.
        // This avoids relying on rusqlite backup APIs which may not be available
        // in the currently used rusqlite version.
        // Remove any existing target file so VACUUM INTO can create it cleanly.
        let _ = std::fs::remove_file(path);
        let safe_path = path.replace("'", "''");
        let sql = format!("VACUUM INTO '{}'", safe_path);
        self.conn
            .execute_batch(&sql)
            .context(format!("Failed to save database to {}", path))?;

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DocpackStats {
    pub file_count: u32,
    pub chunk_count: u32,
    pub embedding_count: u32,
    pub symbol_count: u32,
}
