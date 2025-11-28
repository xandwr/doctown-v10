use doctown_v10::{
    DEFAULT_MAX_TOKENS, EmbeddingClient, ParserRegistry, SandboxBuilder, SandboxError,
    chunk_semantic_units, kmeans,
};
use std::time::Instant;

fn main() -> Result<(), SandboxError> {
    let start_time = Instant::now();
    println!("=== DocTown v10: Sandboxed ZIP Ingestion with Parser Pipeline ===\n");

    // Step 1: Build sandbox from GitHub repo
    let step1_start = Instant::now();
    println!("Step 1: Ingesting repository...");
    let sandbox = SandboxBuilder::new()
        .max_file_size(10 * 1024 * 1024) // 10 MB per file
        .max_total_size(150 * 1024 * 1024) // 150 MB total
        .ingest_github_repo("serde-rs", "serde", "master")?
        .build();

    let step1_duration = step1_start.elapsed();
    println!(
        "✓ Loaded {} files ({} bytes total) [{:.2}s]\n",
        sandbox.file_count(),
        sandbox.total_size(),
        step1_duration.as_secs_f64()
    );

    // Step 2: Create parser registry
    let step2_start = Instant::now();
    println!("Step 2: Initializing parser registry...");
    let registry = ParserRegistry::new();
    let step2_duration = step2_start.elapsed();
    println!(
        "✓ Registry created with fallback parser [{:.2}s]\n",
        step2_duration.as_secs_f64()
    );

    // Step 3: Process all files through parser pipeline
    let step3_start = Instant::now();
    println!("Step 3: Processing files through parser pipeline...\n");
    let mut total_semantic_units = 0;
    let mut total_normalized_bytes = 0;
    let mut utf8_count = 0;
    let mut binary_count = 0;
    let mut all_parse_results = Vec::new();

    for file_entry in sandbox.list() {
        let bytes = sandbox.get(&file_entry.virtual_path).unwrap();
        let parser = registry.select(&file_entry.virtual_path);
        let result = parser.parse(&file_entry.virtual_path, bytes);

        total_semantic_units += result.semantic_units.len();
        total_normalized_bytes += result.normalized_text.len();

        if result.metadata.is_utf8 {
            utf8_count += 1;
        } else {
            binary_count += 1;
        }

        // Show first few files as examples
        if utf8_count + binary_count <= 5 {
            println!(
                "  {} [{}] - {} units, {} lines, {} bytes",
                result.metadata.path,
                result.metadata.language,
                result.semantic_units.len(),
                result.metadata.line_count,
                result.metadata.size_bytes
            );
        }

        all_parse_results.push(result);
    }

    let step3_duration = step3_start.elapsed();
    println!(
        "\n✓ Parsing complete [{:.2}s]\n",
        step3_duration.as_secs_f64()
    );

    // Step 4: Chunk semantic units
    let step4_start = Instant::now();
    println!("Step 4: Chunking semantic units...\n");
    let mut total_chunks = 0;
    let mut total_chunk_tokens = 0;
    let mut chunks_shown = 0;
    let mut all_chunks = Vec::new();

    for parse_result in all_parse_results {
        let chunks = chunk_semantic_units(parse_result.semantic_units, DEFAULT_MAX_TOKENS);

        // Show first few chunked files
        if chunks_shown < 5 && !chunks.is_empty() {
            println!(
                "  {} - {} chunks, avg {} tokens/chunk",
                parse_result.metadata.path,
                chunks.len(),
                chunks.iter().map(|c| c.metadata.token_count).sum::<usize>() / chunks.len()
            );
            chunks_shown += 1;
        }

        total_chunks += chunks.len();
        total_chunk_tokens += chunks.iter().map(|c| c.metadata.token_count).sum::<usize>();
        all_chunks.extend(chunks);
    }

    let step4_duration = step4_start.elapsed();
    println!(
        "\n✓ Chunking complete [{:.2}s]\n",
        step4_duration.as_secs_f64()
    );

    // Step 5: Embed chunks
    let step5_start = Instant::now();
    println!("Step 5: Embedding chunks...\n");

    let embedding_client = EmbeddingClient::new("http://localhost:18115");
    let chunk_texts: Vec<String> = all_chunks.iter().map(|c| c.text.clone()).collect();

    println!(
        "  Sending {} chunks to embedding server...",
        chunk_texts.len()
    );
    let embeddings = match embedding_client.embed_chunks_blocking(chunk_texts) {
        Ok(emb) => {
            println!("  ✓ Received {} embeddings", emb.len());
            if !emb.is_empty() {
                println!("  Embedding dimensions: {}", emb[0].len());
            }
            emb
        }
        Err(e) => {
            eprintln!("  ✗ Embedding failed: {}", e);
            eprintln!("\n  Make sure the Python embedding server is running:");
            eprintln!("    cd python/embedding && python server.py\n");
            return Ok(());
        }
    };

    let step5_duration = step5_start.elapsed();
    println!(
        "\n✓ Embedding complete [{:.2}s]\n",
        step5_duration.as_secs_f64()
    );

    // Step 6: Cluster embeddings
    let step6_start = Instant::now();
    println!("Step 6: Clustering embeddings...\n");

    // Calculate number of clusters (heuristic: sqrt(n) or max 50)
    let k = (embeddings.len() as f64).sqrt().ceil() as usize;
    let k = k.min(50).max(2);

    println!("  Running k-means with k={} clusters...", k);
    let cluster_result = kmeans(&embeddings, k, 100, 42);

    println!("  ✓ Converged in {} iterations", cluster_result.iterations);
    println!("  Total clusters: {}", cluster_result.clusters.len());

    // Show cluster size distribution
    let mut cluster_sizes: Vec<(u32, usize)> = cluster_result
        .clusters
        .iter()
        .map(|c| (c.id, c.chunk_ids.len()))
        .collect();
    cluster_sizes.sort_by_key(|(_id, size)| std::cmp::Reverse(*size));

    println!("\n  Largest clusters:");
    for (id, size) in cluster_sizes.iter().take(5) {
        println!("    Cluster {}: {} chunks", id, size);
    }

    let step6_duration = step6_start.elapsed();
    println!(
        "\n✓ Clustering complete [{:.2}s]\n",
        step6_duration.as_secs_f64()
    );

    // Statistics
    println!("=== Pipeline Statistics ===");
    println!("Total files:          {}", sandbox.file_count());
    println!("UTF-8 files:          {}", utf8_count);
    println!("Binary files:         {}", binary_count);
    println!("Semantic units:       {}", total_semantic_units);
    println!("Normalized bytes:     {}", total_normalized_bytes);
    println!(
        "Avg units/file:       {:.1}",
        total_semantic_units as f64 / sandbox.file_count() as f64
    );
    println!("Total chunks:         {}", total_chunks);
    println!("Total tokens:         {}", total_chunk_tokens);
    println!(
        "Avg tokens/chunk:     {:.1}",
        if total_chunks > 0 {
            total_chunk_tokens as f64 / total_chunks as f64
        } else {
            0.0
        }
    );
    println!("Embeddings:           {}", embeddings.len());
    println!(
        "Embedding dims:       {}",
        if !embeddings.is_empty() {
            embeddings[0].len()
        } else {
            0
        }
    );
    println!("Clusters:             {}", cluster_result.clusters.len());
    println!(
        "Avg chunks/cluster:   {:.1}",
        if cluster_result.clusters.len() > 0 {
            total_chunks as f64 / cluster_result.clusters.len() as f64
        } else {
            0.0
        }
    );

    // Timing breakdown
    let total_duration = start_time.elapsed();
    println!("\n=== Timing Breakdown ===");
    println!(
        "Step 1 (Ingestion):   {:.3}s ({:.1}%)",
        step1_duration.as_secs_f64(),
        100.0 * step1_duration.as_secs_f64() / total_duration.as_secs_f64()
    );
    println!(
        "Step 2 (Registry):    {:.3}s ({:.1}%)",
        step2_duration.as_secs_f64(),
        100.0 * step2_duration.as_secs_f64() / total_duration.as_secs_f64()
    );
    println!(
        "Step 3 (Parsing):     {:.3}s ({:.1}%)",
        step3_duration.as_secs_f64(),
        100.0 * step3_duration.as_secs_f64() / total_duration.as_secs_f64()
    );
    println!(
        "Step 4 (Chunking):    {:.3}s ({:.1}%)",
        step4_duration.as_secs_f64(),
        100.0 * step4_duration.as_secs_f64() / total_duration.as_secs_f64()
    );
    println!(
        "Step 5 (Embedding):   {:.3}s ({:.1}%)",
        step5_duration.as_secs_f64(),
        100.0 * step5_duration.as_secs_f64() / total_duration.as_secs_f64()
    );
    println!(
        "Step 6 (Clustering):  {:.3}s ({:.1}%)",
        step6_duration.as_secs_f64(),
        100.0 * step6_duration.as_secs_f64() / total_duration.as_secs_f64()
    );
    println!("─────────────────────────────────");
    println!("Total execution:      {:.3}s", total_duration.as_secs_f64());

    println!("\n=== System Extensibility ===");
    println!(
        "Current parsers:      {} (fallback only)",
        registry.parser_count()
    );
    println!("Ready for:            Rust, Python, Markdown, JSON, etc.");
    println!(
        "Chunker configured:   Max {} tokens per chunk",
        DEFAULT_MAX_TOKENS
    );
    println!("Embedding model:      google/embeddinggemma-300m (768-dim)");
    println!("Clustering:           K-means with cosine distance");
    println!("\nNext step: Generate summaries from clusters for RAG");

    Ok(())
}
