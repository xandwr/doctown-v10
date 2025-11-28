# Embedder Usage Example

This shows how Rust orchestrates the embedding pipeline while Python does the heavy lifting.

## Architecture Overview

```
┌─────────────────────────────────────────┐
│         Rust (Orchestrator)             │
│                                         │
│  ┌──────────┐    ┌─────────┐           │
│  │ Chunker  │───▶│ Batcher │           │
│  └──────────┘    └─────────┘           │
│                       │                 │
│                       ▼                 │
│              ┌──────────────┐           │
│              │ EmbeddingClient         │
│              └──────────────┘           │
│                       │                 │
└───────────────────────┼─────────────────┘
                        │ HTTP POST
                        ▼
            ┌───────────────────────┐
            │   Python Server       │
            │  (sentence_transformers)
            │                       │
            │  ┌─────────────┐     │
            │  │   Model     │     │
            │  │ (GPU/CPU)   │     │
            │  └─────────────┘     │
            └───────────────────────┘
```

## Step 1: Start Python Server

```bash
cd python/embedding
pip install -r requirements.txt
python server.py
```

The server will:
- Load `google/embeddinggemma-300m` on GPU (if available) or CPU
- Listen on `http://localhost:18115`
- Stay alive and keep the model warm

## Step 2: Use from Rust

```rust
use doctown_v10::{EmbeddingClient, Batcher, EmbeddingModelInfo};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Create client
    let client = EmbeddingClient::new("http://localhost:18115");
    
    // 2. Get model metadata
    let model = EmbeddingModelInfo::default(); // gemma-300m, 768 dims, batch 32
    
    // 3. Create batcher
    let batcher = Batcher::new(model.max_batch);
    
    // 4. Generate your chunk texts (from parser/chunker)
    let chunks = vec![
        "fn main() { println!(\"hello\"); }".to_string(),
        "pub struct User { name: String }".to_string(),
        "impl User { fn new(name: String) -> Self { ... } }".to_string(),
        // ... hundreds or thousands more
    ];
    
    // 5. Batch and embed
    let mut all_embeddings = Vec::new();
    
    for batch in batcher.split(&chunks) {
        let batch_texts = batch.iter().map(|s| s.to_string()).collect();
        let embeddings = client.embed(batch_texts).await?;
        all_embeddings.extend(embeddings);
    }
    
    // 6. Now all_embeddings[i] corresponds to chunks[i]
    println!("Embedded {} chunks", all_embeddings.len());
    println!("Each vector has {} dimensions", all_embeddings[0].len());
    
    Ok(())
}
```

## Step 3: Integration with Chunker

```rust
use doctown_v10::{
    chunk_semantic_units, 
    EmbeddingClient, 
    Batcher, 
    EmbeddingModelInfo,
    SemanticUnit
};

async fn embed_file(units: Vec<SemanticUnit>) -> Result<Vec<(String, Vec<f32>)>, Box<dyn std::error::Error>> {
    // Chunk the units
    let chunks = chunk_semantic_units(&units);
    
    // Extract text
    let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
    
    // Set up embedding
    let client = EmbeddingClient::new("http://localhost:18115");
    let model = EmbeddingModelInfo::default();
    let batcher = Batcher::new(model.max_batch);
    
    // Embed in batches
    let mut all_embeddings = Vec::new();
    
    for batch in batcher.split(&texts) {
        let batch_texts = batch.iter().map(|s| s.to_string()).collect();
        let embeddings = client.embed(batch_texts).await?;
        all_embeddings.extend(embeddings);
    }
    
    // Zip text with embeddings
    let result: Vec<(String, Vec<f32>)> = texts
        .into_iter()
        .zip(all_embeddings)
        .collect();
    
    Ok(result)
}
```

## Error Handling

```rust
use doctown_v10::{EmbeddingClient, EmbedError};

async fn safe_embed(texts: Vec<String>) -> Result<Vec<Vec<f32>>, EmbedError> {
    let client = EmbeddingClient::new("http://localhost:18115");
    
    match client.embed(texts).await {
        Ok(embeddings) => {
            println!("Success! Got {} embeddings", embeddings.len());
            Ok(embeddings)
        }
        Err(EmbedError::ServerError { status, body }) => {
            eprintln!("Server error {}: {}", status, body);
            Err(EmbedError::ServerError { status, body })
        }
        Err(EmbedError::RequestFailed(e)) => {
            eprintln!("Network error: {}", e);
            eprintln!("Is the Python server running?");
            Err(EmbedError::RequestFailed(e))
        }
        Err(e) => {
            eprintln!("Other error: {}", e);
            Err(e)
        }
    }
}
```

## Configuration

### Custom Model

Edit `python/embedding/model.py`:

```python
class EmbeddingModel:
    def __init__(self, model_name="sentence-transformers/all-MiniLM-L6-v2"):
        # Your custom model here
```

Then update the Rust side:

```rust
let model = EmbeddingModelInfo::new("all-MiniLM-L6-v2", 384, 64);
```

### Custom Timeout

```rust
use std::time::Duration;

let client = EmbeddingClient::with_timeout(
    "http://localhost:18115",
    Duration::from_secs(300) // 5 minutes
);
```

## Testing

Run unit tests:
```bash
cargo test --lib
```

Run integration tests (requires Python server running):
```bash
python python/embedding/server.py &
cargo test --lib -- --ignored --test-threads=1
```

## Performance Tips

1. **Batch Size**: Larger batches = better GPU utilization, but may OOM
2. **Keep Server Running**: Model load time is ~10s, so don't restart
3. **Network Overhead**: Run Python on same machine to minimize latency
4. **Parallel Batches**: You can run multiple batches in parallel if your GPU has enough VRAM

## Full Pipeline Example

```rust
use doctown_v10::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // 1. Parse files
    let registry = ParserRegistry::new();
    let parser = registry.get_parser("file.rs")?;
    let content = std::fs::read_to_string("file.rs")?;
    let units = parser.parse(&content)?;
    
    // 2. Chunk
    let chunks = chunk_semantic_units(&units);
    let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
    
    // 3. Embed
    let client = EmbeddingClient::new("http://localhost:18115");
    let model = EmbeddingModelInfo::default();
    let batcher = Batcher::new(model.max_batch);
    
    let mut embeddings = Vec::new();
    for batch in batcher.split(&texts) {
        let batch_texts = batch.iter().map(|s| s.to_string()).collect();
        let vecs = client.embed(batch_texts).await?;
        embeddings.extend(vecs);
    }
    
    // 4. Store or cluster (next phase)
    println!("Embedded {} chunks into {}-dimensional space", 
             embeddings.len(), 
             embeddings[0].len());
    
    Ok(())
}
```
