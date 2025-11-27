## Target RAG flow:

1. **Rust**: chunk
2. **Python**: generate embeddings (tiny Qwen model)
3. **Python**: rerank (0.6B reranker)
4. **Rust**: assemble prompt/cluster
5. **Docker**: run 30B model with the prepared context
6. **Rust**: handle output, create docpack

## Models:

- **Embeddings**: Qwen3-Embedding-0.6B-GGUF
- **Reranker**: Qwen3-Reranker-0.6B
- **Generator**: Qwen3-Coder-30B-A3B-Instruct

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
