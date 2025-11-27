mod chunker;
mod db;
mod docpack;
mod python_bridge;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::path::Path;

use chunker::process_repository;
use docpack::DocpackWriter;
use python_bridge::generate_embeddings;

#[derive(Parser)]
#[command(name = "doctown")]
#[command(about = "Generate docpacks from code repositories", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Build a docpack from a repository
    Build {
        /// Path to the repository
        #[arg(short, long)]
        repo: String,

        /// Output path for the docpack (default: <repo>.docpack)
        #[arg(short, long)]
        output: Option<String>,

        /// Python executable path (default: /opt/venv/bin/python3)
        #[arg(long, default_value = "/opt/venv/bin/python3")]
        python: String,

        /// Embedding model name
        #[arg(long, default_value = "sentence-transformers/all-MiniLM-L6-v2")]
        embedding_model: String,

        /// Skip embedding generation
        #[arg(long)]
        skip_embeddings: bool,
    },

    /// Query an existing docpack
    Query {
        /// Path to the docpack
        #[arg(short, long)]
        docpack: String,

        /// Query string
        #[arg(short, long)]
        query: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("\n❌ ERROR: {}", e);
        
        // Print full error chain
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("  ↳ Caused by: {}", err);
            source = err.source();
        }
        
        std::process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Build {
            repo,
            output,
            python,
            embedding_model,
            skip_embeddings,
        } => {
            build_docpack(&repo, output.as_deref(), &python, &embedding_model, skip_embeddings)?;
        }
        Commands::Query { docpack, query } => {
            query_docpack(&docpack, &query)?;
        }
    }

    Ok(())
}

fn build_docpack(
    repo_path: &str,
    output_path: Option<&str>,
    python_path: &str,
    embedding_model: &str,
    skip_embeddings: bool,
) -> Result<()> {
    eprintln!("\n═══════════════════════════════════════════");
    eprintln!("  DOCTOWN v{}", env!("CARGO_PKG_VERSION"));
    eprintln!("═══════════════════════════════════════════\n");

    // Validate repository path
    if !Path::new(repo_path).exists() {
        anyhow::bail!("Repository path does not exist: {}", repo_path);
    }

    // Determine output path
    let output = match output_path {
        Some(p) => p.to_string(),
        None => {
            let repo_name = Path::new(repo_path)
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("repo");
            format!("{}.docpack", repo_name)
        }
    };

    eprintln!("📦 Building docpack");
    eprintln!("   Source: {}", repo_path);
    eprintln!("   Output: {}", output);
    eprintln!("   Model: {}", embedding_model);
    eprintln!();

    // Step 1: Process repository and chunk files
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("STEP 1: Chunking repository");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    let (file_infos, chunks) = process_repository(repo_path)
        .context("Failed to process repository")?;

    // Step 2: Create docpack and insert data
    eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("STEP 2: Creating docpack database");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    let mut docpack = DocpackWriter::new(
        None, // source_repo
        Some(repo_path.to_string()),
        embedding_model.to_string(),
    )
    .context("Failed to create docpack writer")?;

    // Insert files
    eprintln!("[docpack] Inserting {} files...", file_infos.len());
    for file_info in &file_infos {
        docpack
            .db_mut()
            .insert_file(file_info)
            .context(format!("Failed to insert file: {}", file_info.path))?;
    }

    // Insert chunks
    eprintln!("[docpack] Inserting {} chunks...", chunks.len());
    for chunk in &chunks {
        docpack
            .db_mut()
            .insert_chunk(chunk)
            .context(format!("Failed to insert chunk: {}", chunk.id))?;
    }

    // Step 3: Generate embeddings (optional)
    if !skip_embeddings {
        eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("STEP 3: Generating embeddings");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        
        // Try /opt/embed_chunks.py first (Docker), then local
        let embed_script = if Path::new("/opt/embed_chunks.py").exists() {
            "/opt/embed_chunks.py"
        } else {
            "embed_chunks.py"
        };
        
        // Check if Python and script exist
        if !Path::new(python_path).exists() {
            eprintln!("\n⚠️  WARNING: Python not found at {}", python_path);
            eprintln!("   Skipping embedding generation.");
            eprintln!("   To generate embeddings, ensure Python is installed with sentence-transformers.");
        } else if !Path::new(embed_script).exists() {
            eprintln!("\n⚠️  WARNING: Embedding script not found: {}", embed_script);
            eprintln!("   Skipping embedding generation.");
        } else {
            let embeddings = generate_embeddings(&chunks, python_path, embed_script, embedding_model)
                .context("Failed to generate embeddings")?;

            eprintln!("[docpack] Inserting {} embeddings...", embeddings.len());
            for embedding in &embeddings {
                docpack
                    .db_mut()
                    .insert_embedding(embedding)
                    .context(format!("Failed to insert embedding for chunk: {}", embedding.chunk_id))?;
            }
        }
    } else {
        eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("STEP 3: Skipping embeddings (--skip-embeddings)");
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    }

    // Step 4: Write docpack file
    eprintln!("\n━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    eprintln!("STEP 4: Writing docpack archive");
    eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    
    docpack
        .write_to_file(&output)
        .context("Failed to write docpack file")?;

    // Also save a copy to ~/.localdoc/docpacks/
    if let Some(home_dir) = std::env::var_os("HOME") {
        let local_docpacks_dir = Path::new(&home_dir).join(".localdoc").join("docpacks");
        if let Err(e) = std::fs::create_dir_all(&local_docpacks_dir) {
            eprintln!("\n⚠️  WARNING: Could not create ~/.localdoc/docpacks/: {}", e);
        } else {
            let output_path = Path::new(&output);
            if let Some(filename) = output_path.file_name() {
                let local_copy = local_docpacks_dir.join(filename);
                if let Err(e) = std::fs::copy(&output, &local_copy) {
                    eprintln!("\n⚠️  WARNING: Could not copy to ~/.localdoc/docpacks/: {}", e);
                } else {
                    eprintln!("[docpack] ✓ Saved copy to: {}", local_copy.display());
                }
            }
        }
    }

    eprintln!("\n═══════════════════════════════════════════");
    eprintln!("✅ DOCPACK CREATED SUCCESSFULLY!");
    eprintln!("═══════════════════════════════════════════");
    eprintln!("   Output: {}", output);
    eprintln!();

    Ok(())
}

fn query_docpack(docpack_path: &str, query: &str) -> Result<()> {
    eprintln!("\n═══════════════════════════════════════════");
    eprintln!("  DOCTOWN QUERY");
    eprintln!("═══════════════════════════════════════════\n");

    eprintln!("📦 Loading docpack: {}", docpack_path);
    eprintln!("🔍 Query: {}", query);
    eprintln!();

    let reader = docpack::DocpackReader::open(docpack_path)
        .context("Failed to open docpack")?;

    // Get all chunks
    let chunks = reader.db().get_all_chunks()
        .context("Failed to retrieve chunks")?;

    eprintln!("Found {} chunks in docpack", chunks.len());
    
    // Simple text search (can be enhanced with embeddings later)
    let query_lower = query.to_lowercase();
    let mut matches = Vec::new();

    for chunk in &chunks {
        if chunk.content.to_lowercase().contains(&query_lower) {
            matches.push(chunk);
        }
    }

    eprintln!("Found {} matching chunks\n", matches.len());

    // Display top matches
    for (i, chunk) in matches.iter().take(5).enumerate() {
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        eprintln!("Match {}: {} (lines {}-{})", 
                  i + 1, chunk.file_path, chunk.start_line, chunk.end_line);
        eprintln!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        let preview = if chunk.content.len() > 300 {
            format!("{}...", &chunk.content[..300])
        } else {
            chunk.content.clone()
        };
        eprintln!("{}\n", preview);
    }

    Ok(())
}
