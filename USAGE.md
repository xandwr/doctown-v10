# Doctown Usage Guide

## Quick Start

### Building Locally

```bash
# Build the Rust binary
cargo build --release

# Run on a repository
./target/release/doctown build --repo /path/to/repo --output myrepo.docpack
```

### Using Docker

```bash
# Build the Docker image
docker build -t doctown:latest .

# Or use the build script
chmod +x build.sh
./build.sh

# Run on a repository (mount as volume)
docker run --rm \
  -v /path/to/repo:/repo:ro \
  -v $(pwd):/output \
  doctown:latest build --repo /repo --output /output/myrepo.docpack

# With GPU support for embeddings
docker run --rm --gpus all \
  -v /path/to/repo:/repo:ro \
  -v $(pwd):/output \
  doctown:latest build --repo /repo --output /output/myrepo.docpack
```

## Command Line Interface

### Build Command

Create a docpack from a repository:

```bash
doctown build --repo <PATH> [OPTIONS]

Options:
  -r, --repo <PATH>              Path to the repository (required)
  -o, --output <PATH>            Output path for the docpack (default: <repo>.docpack)
  --python <PATH>                Python executable path (default: /opt/venv/bin/python3)
  --embedding-model <MODEL>      Embedding model name (default: sentence-transformers/all-MiniLM-L6-v2)
  --skip-embeddings              Skip embedding generation
```

Examples:

```bash
# Basic usage
doctown build --repo ./my-project

# Custom output path
doctown build --repo ./my-project --output my-custom-name.docpack

# Skip embeddings (faster, for testing)
doctown build --repo ./my-project --skip-embeddings

# Use custom embedding model
doctown build --repo ./my-project --embedding-model sentence-transformers/paraphrase-MiniLM-L6-v2
```

### Query Command

Query an existing docpack:

```bash
doctown query --docpack <PATH> --query <QUERY>

Options:
  -d, --docpack <PATH>   Path to the docpack (required)
  -q, --query <QUERY>    Query string (required)
```

Example:

```bash
doctown query --docpack my-project.docpack --query "authentication function"
```

## Docpack Structure

A `.docpack` file is a ZIP archive containing:

```
myrepo.docpack
├── docpack.sqlite         # SQLite database with all structured data
│   ├── files              # File metadata (path, hash, size, language)
│   ├── chunks             # Code chunks with line ranges
│   ├── embeddings         # Vector embeddings for chunks
│   └── symbols            # Extracted symbols (functions, classes, etc.)
├── manifest.json          # Human-readable metadata
├── assets/                # Optional: screenshots, diagrams
└── readme.md              # Auto-generated summary
```

## Python Integration

The tool uses Python for ML tasks via subprocess calls:

1. **embed_chunks.py**: Generates embeddings using sentence-transformers
2. **rerank.py**: Reranks chunks using cross-encoder models

These scripts read JSON from stdin and write JSON to stdout, making them:
- Easy to test independently
- Language-agnostic (can be replaced with any implementation)
- Transparent (all data flows through stdio)

### Testing Python Scripts Directly

```bash
# Test embedding script
echo '[{"id":"1","content":"def hello(): pass"}]' | \
  python3 embed_chunks.py --model sentence-transformers/all-MiniLM-L6-v2

# Test reranking script
echo '[{"query":"hello","chunk_id":"1","content":"def hello(): pass"}]' | \
  python3 rerank.py --model cross-encoder/ms-marco-MiniLM-L-6-v2
```

## Docker Deployment

### Building for Production

```bash
# Build with specific CUDA version
docker build \
  --build-arg CUDA_VERSION=12.1.1 \
  -t doctown:prod .

# Multi-platform build
docker buildx build \
  --platform linux/amd64,linux/arm64 \
  -t doctown:latest .
```

### Running in Production

```bash
# With GPU support (NVIDIA)
docker run --rm --gpus all \
  -v /data/repos:/repos:ro \
  -v /data/output:/output \
  doctown:latest build \
    --repo /repos/myproject \
    --output /output/myproject.docpack

# CPU only (slower)
docker run --rm \
  -v /data/repos:/repos:ro \
  -v /data/output:/output \
  doctown:latest build \
    --repo /repos/myproject \
    --output /output/myproject.docpack \
    --skip-embeddings
```

### Environment Variables

- `CUDA_VISIBLE_DEVICES`: Control GPU visibility (default: 0)
- `PYTHONUNBUFFERED`: Ensure real-time Python output (default: 1)

## Development

### Local Development Setup

```bash
# Install Rust dependencies
cargo build

# Create Python virtual environment
python3 -m venv venv
source venv/bin/activate  # or `venv\Scripts\activate` on Windows

# Install Python dependencies
pip install -r requirements.txt

# Run tests
cargo test
```

### Testing

```bash
# Test on this repository
cargo run -- build --repo . --output doctown-self.docpack

# Test query
cargo run -- query --docpack doctown-self.docpack --query "database"
```

## Troubleshooting

### Common Issues

**Error: "Python not found"**
- Ensure Python is installed and accessible
- Use `--python` flag to specify custom Python path
- In Docker, Python is at `/opt/venv/bin/python3`

**Error: "Failed to load model"**
- Check internet connection (models download on first use)
- Verify model name is correct
- For offline use, pre-download models to cache

**Error: "Failed to chunk file"**
- File might be binary or corrupted
- Check file encoding (UTF-8 expected)
- File will be skipped automatically

**Out of memory**
- Use `--skip-embeddings` to reduce memory usage
- Process smaller repositories
- Increase Docker memory limit

### Debugging

Enable verbose output:

```bash
# Rust (via RUST_LOG)
RUST_LOG=debug cargo run -- build --repo .

# Python (stderr is automatically captured)
# Check output after "Script output:" lines
```

## Performance Tips

1. **Use GPU**: Embeddings are 10-100x faster with GPU
2. **Skip embeddings**: Use `--skip-embeddings` for testing/debugging
3. **Batch processing**: Process multiple repos in parallel with Docker
4. **Model size**: Smaller models = faster processing (but lower quality)

## Next Steps

- Add AST-based chunking for better code understanding
- Implement vector similarity search for queries
- Add support for more embedding models (GGUF, etc.)
- Create web UI for docpack exploration
- Add incremental updates (diff-based docpack generation)
