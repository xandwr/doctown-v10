#!/usr/bin/env python3
"""
LLM-based generation script for doctown.
Uses local Qwen3-4B-Thinking model via llama-cpp-python to generate structured documentation.
Reads preprocessed data (chunks, embeddings, symbols) from stdin as JSON.
Outputs structured, enriched documentation as JSON to stdout.
"""

import sys
import json
import argparse
from typing import List, Dict, Any, Optional

def eprint(*args, **kwargs):
    """Print to stderr for logging"""
    print(*args, file=sys.stderr, **kwargs)

# Prompt template for structured output
GENERATION_PROMPT = """You are a code documentation expert. Analyze the provided code structure and generate comprehensive, structured documentation.

INPUT DATA:
{input_data}

Your task:
1. Identify key subsystems/modules and their relationships
2. Generate clear documentation for important symbols (functions, classes, etc.)
3. Provide usage examples where appropriate
4. Identify design patterns and architectural insights
5. Note any potential issues or improvements

OUTPUT FORMAT (strict JSON):
{{
  "subsystems": [
    {{
      "name": "subsystem_name",
      "description": "brief description",
      "confidence": 0.0-1.0,
      "files": ["file1.rs", "file2.rs"],
      "primary_purpose": "explanation"
    }}
  ],
  "enriched_symbols": [
    {{
      "symbol_id": "uuid",
      "name": "symbol_name",
      "documentation": "generated documentation",
      "usage_examples": ["example1", "example2"],
      "related_symbols": ["symbol1", "symbol2"],
      "complexity_notes": "analysis of complexity"
    }}
  ],
  "architecture_insights": [
    {{
      "category": "pattern|design|concern",
      "description": "insight description",
      "affected_components": ["comp1", "comp2"]
    }}
  ],
  "quickstart": {{
    "entry_points": ["main.rs", "lib.rs"],
    "core_types": ["Type1", "Type2"],
    "getting_started": "brief guide"
  }}
}}

IMPORTANT: Output ONLY valid JSON. No markdown, no explanations, just the JSON object."""

def prepare_input_summary(data: Dict[str, Any]) -> str:
    """Prepare a concise summary of input data for the prompt"""
    chunks = data.get("chunks", [])
    symbols = data.get("symbols", [])
    files = data.get("files", [])
    
    # Create a structured summary
    summary = {
        "file_count": len(files),
        "chunk_count": len(chunks),
        "symbol_count": len(symbols),
        "languages": list(set(f.get("language", "unknown") for f in files)),
        "sample_files": [f.get("path", "") for f in files[:10]],
        "sample_symbols": [
            {
                "name": s.get("name", ""),
                "kind": s.get("kind", ""),
                "file": s.get("file_path", "")
            }
            for s in symbols[:20]
        ],
        "sample_chunks": [
            {
                "file": c.get("file_path", ""),
                "type": c.get("chunk_type", ""),
                "name": c.get("name", ""),
                "content_preview": c.get("content", "")[:200]
            }
            for c in chunks[:15]
        ]
    }
    
    return json.dumps(summary, indent=2)

def generate_with_llm(
    model,
    input_data: Dict[str, Any],
    max_tokens: int = 4096,
    temperature: float = 0.3
) -> Dict[str, Any]:
    """Generate structured output using the LLM"""
    
    # Prepare input summary
    input_summary = prepare_input_summary(input_data)
    
    # Format prompt
    prompt = GENERATION_PROMPT.format(input_data=input_summary)
    
    eprint(f"[generate] Prompting LLM with {len(prompt)} characters...")
    
    # Generate response
    response = model(
        prompt,
        max_tokens=max_tokens,
        temperature=temperature,
        stop=None,
        echo=False
    )
    
    # Extract generated text
    generated_text = response.get("choices", [{}])[0].get("text", "").strip()
    
    eprint(f"[generate] Generated {len(generated_text)} characters")
    
    # Parse JSON output
    try:
        # Try to find JSON in the response (in case model adds extra text)
        json_start = generated_text.find("{")
        json_end = generated_text.rfind("}") + 1
        
        if json_start == -1 or json_end == 0:
            raise ValueError("No JSON found in model output")
        
        json_str = generated_text[json_start:json_end]
        result = json.loads(json_str)
        
        # Validate structure
        required_keys = ["subsystems", "enriched_symbols", "architecture_insights", "quickstart"]
        for key in required_keys:
            if key not in result:
                result[key] = [] if key != "quickstart" else {}
        
        return result
        
    except json.JSONDecodeError as e:
        eprint(f"[generate] WARNING: Failed to parse JSON output: {e}")
        eprint(f"[generate] Raw output: {generated_text[:500]}")
        
        # Return minimal valid structure
        return {
            "subsystems": [],
            "enriched_symbols": [],
            "architecture_insights": [],
            "quickstart": {},
            "error": f"Failed to parse JSON: {str(e)}",
            "raw_output": generated_text[:1000]
        }

def main():
    parser = argparse.ArgumentParser(description="Generate structured documentation with LLM")
    parser.add_argument("--model", required=True, help="Path to GGUF model file")
    parser.add_argument("--max-tokens", type=int, default=4096, help="Maximum tokens to generate")
    parser.add_argument("--temperature", type=float, default=0.3, help="Temperature for generation")
    parser.add_argument("--ctx-size", type=int, default=8192, help="Context window size")
    args = parser.parse_args()

    eprint(f"[generate] Loading model: {args.model}")
    eprint(f"[generate] Context size: {args.ctx_size}, Max tokens: {args.max_tokens}")

    try:
        # Import llama-cpp-python
        from llama_cpp import Llama
        
        # Load model
        model = Llama(
            model_path=args.model,
            n_ctx=args.ctx_size,
            n_threads=4,  # Adjust based on available cores
            n_gpu_layers=-1,  # Use GPU if available (set to 0 for CPU only)
            verbose=False
        )
        
        eprint(f"[generate] ✓ Model loaded successfully")
        
    except Exception as e:
        eprint(f"[generate] ERROR: Failed to load model: {e}")
        sys.exit(1)

    # Read input from stdin
    try:
        input_data = sys.stdin.read()
        data = json.loads(input_data)
        eprint(f"[generate] Processing {len(data.get('chunks', []))} chunks, "
               f"{len(data.get('symbols', []))} symbols, "
               f"{len(data.get('files', []))} files")
    except json.JSONDecodeError as e:
        eprint(f"[generate] ERROR: Failed to parse input JSON: {e}")
        sys.exit(1)
    except Exception as e:
        eprint(f"[generate] ERROR: Failed to read input: {e}")
        sys.exit(1)

    # Generate documentation
    try:
        result = generate_with_llm(
            model,
            data,
            max_tokens=args.max_tokens,
            temperature=args.temperature
        )
        
        eprint(f"[generate] ✓ Generated documentation:")
        eprint(f"  - {len(result.get('subsystems', []))} subsystems")
        eprint(f"  - {len(result.get('enriched_symbols', []))} enriched symbols")
        eprint(f"  - {len(result.get('architecture_insights', []))} insights")
        
    except Exception as e:
        eprint(f"[generate] ERROR: Generation failed: {e}")
        sys.exit(1)

    # Output result as JSON to stdout
    try:
        print(json.dumps(result))
    except Exception as e:
        eprint(f"[generate] ERROR: Failed to serialize output: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
