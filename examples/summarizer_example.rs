/// Example: Using the local LLM summarizer
///
/// This example shows how to use the DocumenterClient to summarize
/// code chunks using a local LLM with zero cloud dependencies.
///
/// Prerequisites:
/// 1. Start the Python server: cd python/documenter && python server.py
/// 2. Run this example: cargo run --example summarizer_example
use anyhow::Result;

// These would be imported from your crate
// use doctown::summarizer::client::DocumenterClient;
// For now, we'll show the usage pattern

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Local LLM Summarizer Example ===\n");

    // Example 1: Basic health check
    println!("1. Health Check");
    println!("{}", "-".repeat(50));
    example_health_check().await?;

    // Example 2: Simple summarization
    println!("\n2. Basic Summarization");
    println!("{}", "-".repeat(50));
    example_basic_summarize().await?;

    // Example 3: Custom instructions
    println!("\n3. Custom Instructions");
    println!("{}", "-".repeat(50));
    example_custom_instructions().await?;

    // Example 4: Batch processing
    println!("\n4. Batch Processing");
    println!("{}", "-".repeat(50));
    example_batch().await?;

    Ok(())
}

async fn example_health_check() -> Result<()> {
    // let client = DocumenterClient::new("http://localhost:18116");
    //
    // let health = client.health_check().await?;
    // println!("Status: {}", health.status);
    // println!("Loaded model: {}", health.model);
    // println!("Available models: {:?}", health.available_models);

    println!("✓ Server is healthy");
    println!("✓ Model: qwen3-1.7b");
    println!("✓ Available: [qwen3-1.7b, qwen3-3b, qwen3-7b, ...]");

    Ok(())
}

async fn example_basic_summarize() -> Result<()> {
    let code = r#"
pub struct ChunkMetadata {
    pub file_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub chunk_type: ChunkType,
}

impl ChunkMetadata {
    pub fn new(file_path: String, start_line: usize, end_line: usize) -> Self {
        Self {
            file_path,
            start_line,
            end_line,
            chunk_type: ChunkType::Code,
        }
    }
}
"#;

    println!("Code to summarize:\n{}\n", code);

    // let client = DocumenterClient::new("http://localhost:18116");
    // let summary = client.summarize(
    //     code.to_string(),
    //     None,
    // ).await?;
    //
    // println!("Summary: {}", summary);

    let example_summary = "This code defines a ChunkMetadata struct that stores \
        metadata about code chunks including file path, line numbers, and chunk type. \
        It provides a constructor for easy instantiation with default chunk type.";

    println!("Summary: {}", example_summary);

    Ok(())
}

async fn example_custom_instructions() -> Result<()> {
    let code = r#"
async fn process_file(path: &Path) -> Result<Vec<Chunk>> {
    let content = tokio::fs::read_to_string(path).await?;
    let chunks = chunk_code(&content)?;
    Ok(chunks)
}
"#;

    let instructions = "Explain this function focusing on its async nature and error handling";

    println!("Code:\n{}\n", code);
    println!("Instructions: {}\n", instructions);

    // let client = DocumenterClient::new("http://localhost:18116");
    // let summary = client.summarize(
    //     code.to_string(),
    //     Some(instructions.to_string()),
    // ).await?;
    //
    // println!("Summary: {}", summary);

    let example_summary = "This async function reads a file asynchronously using tokio, \
        processes its content into chunks, and returns them. It uses the ? operator for \
        error propagation, allowing both I/O and chunking errors to bubble up.";

    println!("Summary: {}", example_summary);

    Ok(())
}

async fn example_batch() -> Result<()> {
    let chunks = vec![
        "fn add(a: i32, b: i32) -> i32 { a + b }",
        "fn multiply(a: i32, b: i32) -> i32 { a * b }",
        "fn divide(a: i32, b: i32) -> Option<i32> { if b != 0 { Some(a / b) } else { None } }",
    ];

    println!("Processing {} chunks...\n", chunks.len());

    // let client = DocumenterClient::new("http://localhost:18116");
    //
    // for (i, chunk) in chunks.iter().enumerate() {
    //     let summary = client.summarize(
    //         chunk.to_string(),
    //         Some("Summarize in one sentence".to_string()),
    //     ).await?;
    //
    //     println!("Chunk {}: {}", i + 1, summary);
    // }

    println!("Chunk 1: Returns the sum of two integers");
    println!("Chunk 2: Returns the product of two integers");
    println!("Chunk 3: Safely divides two integers, returning None if divisor is zero");

    println!("\n✓ Processed {} chunks successfully", chunks.len());

    Ok(())
}
