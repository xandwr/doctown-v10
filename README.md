## Current Pipeline:

1. **Rust**: Chunk files into semantic code segments
2. **Python**: Generate embeddings (sentence-transformers)
3. **Python**: Generate documentation with local LLM (Qwen3-4B-Thinking via llama-cpp-python)
4. **Rust**: Assemble and write docpack

### Pipeline Features

✅ **Implemented:**
- File chunking with language detection
- Embedding generation with sentence-transformers
- **Local LLM documentation generation** with structured JSON output
- Database schema for generated content (subsystems, enriched symbols, architecture insights, quickstart)
- Complete integration into build pipeline

⏳ **Planned:**
- Reranking with 0.6B reranker model
- Larger generation models (30B) for complex codebases

## Models:

- **Embeddings**: sentence-transformers (default: all-MiniLM-L6-v2)
- **Generator**: Qwen3-4B-Thinking GGUF model via llama-cpp-python
  - Recommended: [Qwen3-4B-Thinking-2507](https://huggingface.co/Qwen/Qwen3-4B-Thinking-2507) (convert to GGUF)
  - Or any GGUF model compatible with llama-cpp-python
- **Reranker** (planned): Qwen3-Reranker-0.6B

## Usage:

### Basic build with embeddings only:
```bash
doctown build --repo /path/to/repo
```

### Build with LLM generation:
```bash
doctown build \
  --repo /path/to/repo \
  --generator-model /path/to/qwen3-4b-thinking.gguf \
  --max-tokens 4096 \
  --temperature 0.3
```

### Skip specific steps:
```bash
# Skip embeddings
doctown build --repo /path/to/repo --skip-embeddings

# Skip generation
doctown build --repo /path/to/repo --skip-generation
```

### CLI Options:
- `--repo`: Path to repository to process
- `--output`: Output path for docpack (default: `<repo>.docpack`)
- `--python`: Python executable path (default: `/opt/venv/bin/python3`)
- `--embedding-model`: Model for embeddings (default: `sentence-transformers/all-MiniLM-L6-v2`)
- `--generator-model`: Path to GGUF model for LLM generation (optional)
- `--max-tokens`: Max tokens for generation (default: 4096)
- `--temperature`: Temperature for generation (default: 0.3)
- `--skip-embeddings`: Skip embedding generation
- `--skip-generation`: Skip LLM generation

## Deployment:

Target is a single Dockerfile that can execute this entire pipeline in a nice self-contained fashion cleanly and quickly.

Something like:

```dockerfile
FROM nvidia/cuda:12.1.1-devel-ubuntu22.04

# 1. Install system deps
# 2. Install Python
# 3. Create /opt/venv, pip install requirements.txt
# 4. Copy Rust source, build FROM inside container or copy prebuilt artifact
# 5. Copy Python scripts + HF/gguf models
# 6. Expose entrypoint to Rust binary
```

And Rust calls Python like:

```rs
let output = Command::new("/opt/venv/bin/python3")
    .arg("embed_chunks.py")
    .arg(input_path)
    .output()?;
```

Then Python scripts import sentence_transformers, load GGUF models via llama-cpp-python or sglang/qwen_cpp.
Trying to keep the environment fully deterministic and reproducable while leveraging Python's excellent
sentence_transformers OoTB support to not reinvent the wheel.

## End goal:

> `docker run doctown-builder --repo https://github.com/whatever/foo`

- user uploads link/repo
- container spins up
- runs the entire pipeline
- outputs a deterministic docpack
- shuts down

### Notes:

- Rust is the orchestrator
- Python is the semantic layer
- Qwen small models handle cognition
- Qwen3-Coder-30B handles synthesis
- all models are pre-baked
- all deps are deterministic
- one Docker container runs everything
- easy to scale
- easy to debug
- zero ops burden
- portable to anything (RunPod, local, cloud, laptop)
