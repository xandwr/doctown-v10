use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::Write;

use crate::db::{CodeChunk, Embedding, Symbol, FileInfo};

#[derive(Debug, Serialize, Deserialize)]
struct ChunkInput {
    id: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct EmbeddingOutput {
    chunk_id: String,
    vector: Vec<f32>,
    error: Option<String>,
}

/// Call Python script to generate embeddings for chunks
pub fn generate_embeddings(
    chunks: &[CodeChunk],
    python_path: &str,
    script_path: &str,
    model_name: &str,
) -> Result<Vec<Embedding>> {
    eprintln!("[python] Generating embeddings for {} chunks...", chunks.len());
    eprintln!("[python] Using Python: {}", python_path);
    eprintln!("[python] Script: {}", script_path);
    eprintln!("[python] Model: {}", model_name);

    // Prepare input data
    let chunk_inputs: Vec<ChunkInput> = chunks
        .iter()
        .map(|chunk| ChunkInput {
            id: chunk.id.clone(),
            content: chunk.content.clone(),
        })
        .collect();

    let input_json = serde_json::to_string(&chunk_inputs)
        .context("Failed to serialize chunks to JSON")?;

    // Call Python script
    let mut child = Command::new(python_path)
        .arg(script_path)
        .arg("--model")
        .arg(model_name)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn Python process")?;

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input_json.as_bytes())
            .context("Failed to write to Python stdin")?;
    }

    // Wait for completion and collect output
    let output = child
        .wait_with_output()
        .context("Failed to wait for Python process")?;

    // Check exit status
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[python] Error output:\n{}", stderr);
        bail!("Python script failed with exit code: {:?}", output.status.code());
    }

    // Print stderr for debugging
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("[python] Script output:\n{}", stderr);
    }

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let embedding_outputs: Vec<EmbeddingOutput> = serde_json::from_str(&stdout)
        .context(format!("Failed to parse Python output as JSON. Output was:\n{}", stdout))?;

    // Check for errors in output
    let mut embeddings = Vec::new();
    for emb_out in embedding_outputs {
        if let Some(error) = emb_out.error {
            eprintln!("[python] Warning: Failed to embed chunk {}: {}", emb_out.chunk_id, error);
            continue;
        }

        embeddings.push(Embedding {
            chunk_id: emb_out.chunk_id,
            vector: emb_out.vector,
            model: model_name.to_string(),
        });
    }

    eprintln!("[python] ✓ Generated {} embeddings", embeddings.len());

    if embeddings.is_empty() {
        bail!("No embeddings were generated successfully");
    }

    Ok(embeddings)
}

#[derive(Debug, Serialize, Deserialize)]
struct RerankInput {
    query: String,
    chunk_id: String,
    content: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct RerankOutput {
    chunk_id: String,
    score: f32,
    error: Option<String>,
}

/// Call Python script to rerank chunks based on a query
pub fn rerank_chunks(
    query: &str,
    chunks: &[CodeChunk],
    python_path: &str,
    script_path: &str,
    model_name: &str,
) -> Result<Vec<(String, f32)>> {
    eprintln!("[python] Reranking {} chunks...", chunks.len());
    eprintln!("[python] Query: {}", query);

    // Prepare input data
    let rerank_inputs: Vec<RerankInput> = chunks
        .iter()
        .map(|chunk| RerankInput {
            query: query.to_string(),
            chunk_id: chunk.id.clone(),
            content: chunk.content.clone(),
        })
        .collect();

    let input_json = serde_json::to_string(&rerank_inputs)
        .context("Failed to serialize rerank input to JSON")?;

    // Call Python script
    let mut child = Command::new(python_path)
        .arg(script_path)
        .arg("--model")
        .arg(model_name)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn Python process for reranking")?;

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input_json.as_bytes())
            .context("Failed to write to Python stdin")?;
    }

    // Wait for completion and collect output
    let output = child
        .wait_with_output()
        .context("Failed to wait for Python reranking process")?;

    // Check exit status
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[python] Error output:\n{}", stderr);
        bail!("Python reranking script failed with exit code: {:?}", output.status.code());
    }

    // Print stderr for debugging
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("[python] Script output:\n{}", stderr);
    }

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let rerank_outputs: Vec<RerankOutput> = serde_json::from_str(&stdout)
        .context(format!("Failed to parse Python reranking output. Output was:\n{}", stdout))?;

    // Collect results
    let mut results = Vec::new();
    for rerank_out in rerank_outputs {
        if let Some(error) = rerank_out.error {
            eprintln!("[python] Warning: Failed to rerank chunk {}: {}", rerank_out.chunk_id, error);
            continue;
        }

        results.push((rerank_out.chunk_id, rerank_out.score));
    }

    // Sort by score descending
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    eprintln!("[python] ✓ Reranked {} chunks", results.len());

    Ok(results)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationInput {
    pub chunks: Vec<CodeChunk>,
    pub symbols: Vec<Symbol>,
    pub files: Vec<FileInfo>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Subsystem {
    pub name: String,
    pub description: String,
    pub confidence: f32,
    pub files: Vec<String>,
    pub primary_purpose: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EnrichedSymbol {
    pub symbol_id: String,
    pub name: String,
    pub documentation: String,
    pub usage_examples: Vec<String>,
    pub related_symbols: Vec<String>,
    pub complexity_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArchitectureInsight {
    pub category: String,
    pub description: String,
    pub affected_components: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuickstartInfo {
    pub entry_points: Vec<String>,
    pub core_types: Vec<String>,
    pub getting_started: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GenerationOutput {
    pub subsystems: Vec<Subsystem>,
    pub enriched_symbols: Vec<EnrichedSymbol>,
    pub architecture_insights: Vec<ArchitectureInsight>,
    pub quickstart: QuickstartInfo,
    pub error: Option<String>,
    pub raw_output: Option<String>,
}

/// Call Python script to generate documentation with LLM
pub fn generate_documentation(
    chunks: &[CodeChunk],
    symbols: &[Symbol],
    files: &[FileInfo],
    python_path: &str,
    script_path: &str,
    model_path: &str,
    max_tokens: u32,
    temperature: f32,
) -> Result<GenerationOutput> {
    eprintln!("[python] Generating documentation with LLM...");
    eprintln!("[python] Model: {}", model_path);
    eprintln!("[python] Processing {} chunks, {} symbols, {} files", 
              chunks.len(), symbols.len(), files.len());

    // Prepare input data
    let input = GenerationInput {
        chunks: chunks.to_vec(),
        symbols: symbols.to_vec(),
        files: files.to_vec(),
    };

    let input_json = serde_json::to_string(&input)
        .context("Failed to serialize generation input to JSON")?;

    // Call Python script
    let mut child = Command::new(python_path)
        .arg(script_path)
        .arg("--model")
        .arg(model_path)
        .arg("--max-tokens")
        .arg(max_tokens.to_string())
        .arg("--temperature")
        .arg(temperature.to_string())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .context("Failed to spawn Python process for generation")?;

    // Write input to stdin
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(input_json.as_bytes())
            .context("Failed to write to Python stdin")?;
    }

    // Wait for completion and collect output
    let output = child
        .wait_with_output()
        .context("Failed to wait for Python generation process")?;

    // Check exit status
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("[python] Error output:\n{}", stderr);
        bail!("Python generation script failed with exit code: {:?}", output.status.code());
    }

    // Print stderr for debugging
    let stderr = String::from_utf8_lossy(&output.stderr);
    if !stderr.is_empty() {
        eprintln!("[python] Script output:\n{}", stderr);
    }

    // Parse output
    let stdout = String::from_utf8_lossy(&output.stdout);
    let generation_output: GenerationOutput = serde_json::from_str(&stdout)
        .context(format!("Failed to parse Python generation output. Output was:\n{}", stdout))?;

    if let Some(ref error) = generation_output.error {
        eprintln!("[python] Warning: Generation had errors: {}", error);
    }

    eprintln!("[python] ✓ Generated documentation:");
    eprintln!("  - {} subsystems", generation_output.subsystems.len());
    eprintln!("  - {} enriched symbols", generation_output.enriched_symbols.len());
    eprintln!("  - {} architecture insights", generation_output.architecture_insights.len());

    Ok(generation_output)
}
