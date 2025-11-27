use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::process::{Command, Stdio};
use std::io::Write;

use crate::db::{CodeChunk, Embedding};

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
