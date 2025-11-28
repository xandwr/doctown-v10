use doctown_v10::{ParserRegistry, SandboxBuilder, SandboxError};

fn main() -> Result<(), SandboxError> {
    println!("=== DocTown v10: Sandboxed ZIP Ingestion with Parser Pipeline ===\n");

    // Step 1: Build sandbox from GitHub repo (using serde for testing - small repo)
    println!("Step 1: Ingesting repository...");
    let sandbox = SandboxBuilder::new()
        .max_file_size(10 * 1024 * 1024) // 10 MB per file
        .max_total_size(150 * 1024 * 1024) // 150 MB total
        .ingest_github_repo("serde-rs", "serde", "master")?
        .build();

    println!(
        "✓ Loaded {} files ({} bytes total)\n",
        sandbox.file_count(),
        sandbox.total_size()
    );

    // Step 2: Create parser registry (all files use UnknownParser for now)
    println!("Step 2: Initializing parser registry...");
    let registry = ParserRegistry::new();
    println!("✓ Registry created with fallback parser\n");

    // Step 3: Process all files through parser pipeline
    println!("Step 3: Processing files through parser pipeline...\n");
    let mut total_semantic_units = 0;
    let mut total_normalized_bytes = 0;
    let mut utf8_count = 0;
    let mut binary_count = 0;

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
    }

    println!("\n=== Pipeline Statistics ===");
    println!("Total files:          {}", sandbox.file_count());
    println!("UTF-8 files:          {}", utf8_count);
    println!("Binary files:         {}", binary_count);
    println!("Semantic units:       {}", total_semantic_units);
    println!("Normalized bytes:     {}", total_normalized_bytes);
    println!(
        "Avg units/file:       {:.1}",
        total_semantic_units as f64 / sandbox.file_count() as f64
    );

    // Step 4: Demonstrate extensibility
    println!("\n=== System Extensibility ===");
    println!(
        "Current parsers:      {} (fallback only)",
        registry.parser_count()
    );
    println!("Ready for:            Rust, Python, Markdown, JSON, etc.");
    println!("\nNext step: Implement language-specific parsers that produce ParseResult");

    Ok(())
}
