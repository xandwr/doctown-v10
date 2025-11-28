# LLM Summarizer

A **100% local, zero cloud dependency** LLM-based code summarization service following the same Python server pattern as the embedding system.

## Architecture

```
Rust (doctown) ←→ Python FastAPI Server ←→ Local LLM (HuggingFace Transformers)
```

## Key Components

### 1. **Model Registry** ([python/documenter/registry.py](python/documenter/registry.py))
- Centralized configuration for different LLM models
- Each model has specific parameters (temperature, tokens, quantization, etc.)
- Pre-configured models:
  - `qwen3-1.7b` (default, fast)
  - `qwen3-3b` (balanced)
  - `qwen3-7b` (high quality, 8-bit quantized)
  - `phi-3-mini` (Microsoft alternative)
  - `deepseek-coder-1.3b` (code-specialized)

### 2. **Model Wrapper** ([python/documenter/model.py](python/documenter/model.py))
- Loads models from registry
- Handles chat template formatting
- Supports quantization (8-bit/4-bit)
- Auto-detects CUDA/CPU
- Proper prompt formatting per model

### 3. **FastAPI Server** ([python/documenter/server.py](python/documenter/server.py))
- HTTP endpoints: `/summarize`, `/health`, `/models`
- Environment-based configuration
- Proper error handling and validation
- Model loaded on startup

### 4. **Prompts** ([python/documenter/prompts.py](python/documenter/prompts.py))
- Pre-built prompts for different use cases:
  - Chunk summaries
  - Cluster summaries
  - Project overviews
  - Function/Class/Module summaries
- Helper functions to build prompts

### 5. **Rust Client** ([src/summarizer/client.rs](src/summarizer/client.rs))
- HTTP client matching embedder pattern
- Error handling with `SummarizerError`
- Health check support
- 3-minute timeout for LLM generation
- Support for system prompts

## Quick Start

### Install Dependencies
```bash
cd python/documenter
pip install -r requirements.txt
```

### Start Server (Default Model)
```bash
python server.py
```

### Start Server (Custom Model)
```bash
export DOCUMENTER_MODEL=qwen3-3b
python server.py
```

### Test It
```bash
python test_server.py
```

## Usage from Rust

```rust
use crate::summarizer::client::DocumenterClient;

// Create client
let client = DocumenterClient::new("http://localhost:18116");

// Health check
let health = client.health_check().await?;
println!("Model: {}", health.model);

// Summarize
let summary = client.summarize(
    code_chunk,
    Some("Summarize this code".to_string())
).await?;
```

## Environment Variables

```bash
DOCUMENTER_MODEL=qwen3-1.7b   # Which model to load
DOCUMENTER_PORT=18116         # Server port
```

## Adding New Models

Edit `python/documenter/registry.py`:

```python
MODEL_REGISTRY = {
    "my-model": ModelConfig(
        model_id="org/model-name-on-huggingface",
        max_new_tokens=512,
        temperature=0.3,
        do_sample=True,
        supports_system_prompt=True,
        requires_chat_template=True,
        description="My custom model",
    ),
}
```

Then use it:
```bash
export DOCUMENTER_MODEL=my-model
python server.py
```

## Next Steps

1. Start the documenter server
2. Update Rust code to use `DocumenterClient`
3. Use the pre-built prompts from `prompts.py`
4. Adjust model selection based on VRAM/performance needs (aiming for 12gb VRAM)

## Model Selection

| Use Case | Recommended Model | Why |
|----------|------------------|-----|
| Development/Testing | `qwen3-1.7b` | Fast iteration |
| Balanced Quality | `qwen3-3b` | Good quality, reasonable speed |
| Production | `qwen3-7b` | Best quality (needs ~8GB VRAM) |
| Code-heavy | `deepseek-coder-1.3b` | Code-specialized |

---