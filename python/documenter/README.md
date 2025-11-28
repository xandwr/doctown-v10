# Doctown Summarizer - Local LLM Service

Local LLM-based code summarization service with **zero cloud dependencies**.

## Features

- 🏠 **100% Local** - All processing happens on your machine
- 🔌 **Model Registry** - Easy switching between different LLMs
- ⚡ **Fast** - Optimized for code summarization tasks
- 🎯 **Production Ready** - Proper error handling, health checks, and monitoring
- 🔧 **Configurable** - Environment-based configuration

## Quick Start

1. **Install dependencies:**
   ```bash
   cd python/documenter
   pip install -r requirements.txt
   ```

2. **Start the server:**
   ```bash
   python server.py
   ```

3. **Test it:**
   ```bash
   curl -X POST http://localhost:18116/summarize \
     -H "Content-Type: application/json" \
     -d '{"text": "def hello():\n    print(\"world\")"}'
   ```

## Available Models

The registry includes several pre-configured models:

| Model Name | Size | Description | Best For |
|------------|------|-------------|----------|
| `qwen3-1.7b` | 1.5B | **Default** - Fast, efficient | Quick iteration |
| `qwen3-3b` | 3B | Better quality | Balanced performance |
| `qwen3-7b` | 7B | Highest quality | Production use (requires more VRAM) |
| `phi-3-mini` | 3.8B | Microsoft's efficient model | Alternative option |
| `deepseek-coder-1.3b` | 1.3B | Code-specialized | Code-heavy projects |

## Configuration

### Environment Variables

```bash
# Choose which model to load (default: qwen3-1.7b)
export DOCUMENTER_MODEL=qwen3-3b

# Change port (default: 18116)
export DOCUMENTER_PORT=8080

# Start server
python server.py
```

### Adding New Models

Edit [registry.py](./registry.py) to add new models:

```python
MODEL_REGISTRY = {
    "my-model": ModelConfig(
        model_id="org/model-name",
        max_new_tokens=512,
        temperature=0.3,
        description="My custom model",
        # ... other config
    ),
}
```

## API Endpoints

### POST /summarize
Generate a summary of code/text.

**Request:**
```json
{
  "text": "code to summarize",
  "instructions": "optional custom instructions",
  "system_prompt": "optional system prompt override"
}
```

**Response:**
```json
{
  "summary": "Generated summary text"
}
```

### GET /health
Check server health and loaded model.

**Response:**
```json
{
  "status": "healthy",
  "model": "qwen3-1.7b",
  "available_models": ["qwen3-1.7b", "qwen3-3b", ...]
}
```

### GET /models
List all available models in the registry.

## Integration with Rust

The Rust client in `src/summarizer/client.rs` provides a typed interface:

```rust
use crate::summarizer::client::DocumenterClient;

let client = DocumenterClient::new("http://localhost:18116");

// Health check
let health = client.health_check().await?;
println!("Loaded model: {}", health.model);

// Summarize
let summary = client.summarize(
    code_text,
    Some("Explain what this function does".to_string())
).await?;
```

## Prompts

Pre-built prompts are available in [prompts.py](./prompts.py) for:

- **Chunk summaries** - Individual code sections
- **Cluster summaries** - Groups of related chunks
- **Project overview** - High-level architecture
- **Function/Class/Module** - Specific code constructs

## Performance Tips

1. **GPU Acceleration**: Models automatically use CUDA if available
2. **Quantization**: Larger models (7B+) use 8-bit quantization by default
3. **Batch Size**: Adjust in model config for your hardware
4. **Model Selection**: Start with `qwen3-1.7b` for development, use `qwen3-7b` for production

## Troubleshooting

**Server won't start:**
- Check CUDA/GPU drivers if using GPU
- Verify all dependencies installed: `pip install -r requirements.txt`
- Check model name is valid: `python -c "from registry import list_available_models; print(list_available_models())"`

**Out of memory:**
- Use smaller model: `export DOCUMENTER_MODEL=qwen3-1.7b`
- Enable CPU mode by disabling CUDA
- Lower `max_new_tokens` in registry config

**Slow generation:**
- Ensure GPU is being used (check startup logs)
- Try smaller model
- Check `do_sample=False` for faster deterministic output

## Architecture

```
┌─────────────────────┐
│   Rust Client       │
│  (async/await)      │
└──────────┬──────────┘
           │ HTTP/JSON
           ▼
┌─────────────────────┐
│   FastAPI Server    │
│  (server.py)        │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  DocumenterModel    │
│   (model.py)        │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Model Registry     │
│  (registry.py)      │
└─────────────────────┘
           │
           ▼
┌─────────────────────┐
│   Local LLM         │
│  (HuggingFace)      │
└─────────────────────┘
```

## License

Part of the Doctown project.
