use doctown_v10::{
    DEFAULT_MAX_TOKENS, EmbeddingClient, ParserRegistry, SandboxBuilder, SandboxError,
    chunk_semantic_units, kmeans,
};
use std::time::Instant;
use std::process::{Command, Child};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

fn main() -> Result<(), SandboxError> {
    // Track spawned service processes so we can clean them up
    let service_processes = Arc::new(Mutex::new(Vec::<Child>::new()));
    let processes_clone = Arc::clone(&service_processes);
    
    // Register cleanup handler for Ctrl+C
    ctrlc::set_handler(move || {
        eprintln!("\n🛑 Shutting down services...");
        cleanup_services(&processes_clone);
        std::process::exit(0);
    }).expect("Error setting Ctrl-C handler");
    
    // Check and auto-launch backend services if needed
    check_and_launch_services(&service_processes);
    
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

    // Clean up services before exiting
    println!("\n🛑 Shutting down services...");
    cleanup_services(&service_processes);

    Ok(())
}

fn check_and_launch_services(service_processes: &Arc<Mutex<Vec<Child>>>) {
    println!("Checking backend services...");
    
    // First, clean up any existing Python server processes to avoid port conflicts and CUDA memory leaks
    println!("  🧹 Cleaning up existing backend processes...");
    kill_existing_services();
    
    // Give the OS a moment to clean up
    std::thread::sleep(std::time::Duration::from_millis(500));
    
    // Launch embedding service
    println!("  🚀 Launching embedding service...");
    match launch_service("Embedding Service", &["python3", "server.py"], "python/embedding") {
        Ok(child) => {
            service_processes.lock().unwrap().push(child);
            println!("  ⏳ Waiting for embedding service to be ready...");
            wait_for_service("http://localhost:18115/health", "Embedding Service", 60);
        }
        Err(e) => eprintln!("  ✗ Failed to launch embedding service: {}", e),
    }
    
    // Launch documenter service
    println!("  🚀 Launching documenter service...");
    match launch_service("Documenter Service", &["python3", "server.py"], "python/documenter") {
        Ok(child) => {
            service_processes.lock().unwrap().push(child);
            println!("  ⏳ Waiting for documenter service to be ready...");
            wait_for_service("http://localhost:18116/health", "Documenter Service", 60);
        }
        Err(e) => eprintln!("  ✗ Failed to launch documenter service: {}", e),
    }
    
    println!();
}

fn wait_for_service(url: &str, name: &str, timeout_secs: u64) {
    let start = Instant::now();
    let timeout = std::time::Duration::from_secs(timeout_secs);
    
    while start.elapsed() < timeout {
        if check_service(url) {
            println!("  ✓ {} is ready!", name);
            return;
        }
        
        // Show progress every 5 seconds
        let elapsed = start.elapsed().as_secs();
        if elapsed > 0 && elapsed % 5 == 0 {
            println!("    ... still waiting ({:.0}s elapsed)", elapsed);
        }
        
        std::thread::sleep(std::time::Duration::from_millis(500));
    }
    
    eprintln!("  ⚠ {} did not respond within {}s - continuing anyway", name, timeout_secs);
}

fn kill_existing_services() {
    // Kill any Python processes running server.py in embedding or documenter directories
    let _ = Command::new("pkill")
        .arg("-f")
        .arg("python3.*embedding.*server.py")
        .output();
    
    let _ = Command::new("pkill")
        .arg("-f")
        .arg("python3.*documenter.*server.py")
        .output();
}

fn check_service(url: &str) -> bool {
    match reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_millis(500))
        .build()
    {
        Ok(client) => match client.get(url).send() {
            Ok(response) => response.status().is_success(),
            Err(_) => false,
        },
        Err(_) => false,
    }
}

fn launch_service(title: &str, command_args: &[&str], relative_path: &str) -> std::io::Result<Child> {
    let project_root = std::env::current_dir()?;
    let working_dir = project_root.join(relative_path);
    
    launch_in_terminal(title, command_args, &working_dir)
}

fn cleanup_services(service_processes: &Arc<Mutex<Vec<Child>>>) {
    let mut processes = service_processes.lock().unwrap();
    
    // Kill all tracked child processes
    for child in processes.iter_mut() {
        let _ = child.kill();
    }
    
    // Also kill any lingering Python server processes
    kill_existing_services();
    
    processes.clear();
}

fn launch_in_terminal(title: &str, command_args: &[&str], working_dir: &PathBuf) -> std::io::Result<Child> {
    let command_str = command_args.join(" ");
    
    // Get project root to access python/.venv
    let project_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let venv_activate = project_root.join("python").join(".venv").join("bin").join("activate");
    
    // Source bashrc, activate uv venv if it exists, then run command
    // Window will close automatically when the command finishes or is killed (no -hold, no read)
    let full_command = format!(
        "source ~/.bashrc 2>/dev/null || source /etc/bash.bashrc 2>/dev/null; \
         if [ -f '{}' ]; then source '{}'; fi; \
         cd '{}' && {}",
        venv_activate.display(),
        venv_activate.display(),
        working_dir.display(),
        command_str
    );
    
    // Try xterm first (now installed, reliable, no snap conflicts)
    // Remove -hold so window closes when process exits/is killed
    let result = Command::new("xterm")
        .arg("-title")
        .arg(title)
        .arg("-e")
        .arg("bash")
        .arg("-c")
        .arg(&full_command)
        .spawn();
    
    if result.is_ok() {
        return result;
    }
    
    // Try konsole (KDE)
    let result = Command::new("konsole")
        .arg("--title")
        .arg(title)
        .arg("-e")
        .arg("bash")
        .arg("-c")
        .arg(&full_command)
        .spawn();
    
    if result.is_ok() {
        return result;
    }
    
    // Try gnome-terminal with clean env to avoid snap issues
    let result = Command::new("env")
        .arg("-i")
        .arg("DISPLAY=:0")
        .arg(format!("HOME={}", std::env::var("HOME").unwrap_or_else(|_| "/home/xander".to_string())))
        .arg("PATH=/usr/local/bin:/usr/bin:/bin")
        .arg("gnome-terminal")
        .arg("--title")
        .arg(title)
        .arg("--")
        .arg("bash")
        .arg("-c")
        .arg(&full_command)
        .spawn();
    
    if result.is_ok() {
        return result;
    }
    
    // Try x-terminal-emulator as fallback
    Command::new("x-terminal-emulator")
        .arg("-e")
        .arg("bash")
        .arg("-c")
        .arg(&full_command)
        .spawn()
}
