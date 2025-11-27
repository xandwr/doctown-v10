#!/usr/bin/env python3
"""
Embedding generation script for doctown.
Reads chunks from stdin as JSON, generates embeddings, outputs to stdout as JSON.
"""

import sys
import json
import argparse
from typing import List, Dict, Any

def eprint(*args, **kwargs):
    """Print to stderr for logging"""
    print(*args, file=sys.stderr, **kwargs)

def main():
    parser = argparse.ArgumentParser(description="Generate embeddings for code chunks")
    parser.add_argument("--model", required=True, help="Model name to use for embeddings")
    args = parser.parse_args()

    eprint(f"[embed] Loading model: {args.model}")

    try:
        # Import heavy dependencies only after arg parsing
        from sentence_transformers import SentenceTransformer
        
        # Load model
        model = SentenceTransformer(args.model)
        eprint(f"[embed] ✓ Model loaded successfully")
        
    except Exception as e:
        eprint(f"[embed] ERROR: Failed to load model: {e}")
        sys.exit(1)

    # Read input from stdin
    try:
        input_data = sys.stdin.read()
        chunks = json.loads(input_data)
        eprint(f"[embed] Processing {len(chunks)} chunks...")
    except json.JSONDecodeError as e:
        eprint(f"[embed] ERROR: Failed to parse input JSON: {e}")
        sys.exit(1)
    except Exception as e:
        eprint(f"[embed] ERROR: Failed to read input: {e}")
        sys.exit(1)

    # Generate embeddings
    results = []
    batch_size = 32
    
    try:
        for i in range(0, len(chunks), batch_size):
            batch = chunks[i:i + batch_size]
            batch_texts = [chunk["content"] for chunk in batch]
            
            try:
                # Generate embeddings for batch
                embeddings = model.encode(
                    batch_texts,
                    normalize_embeddings=True,
                    show_progress_bar=False,
                    convert_to_numpy=True
                )
                
                # Convert to results
                for j, chunk in enumerate(batch):
                    embedding_list = embeddings[j].tolist()
                    results.append({
                        "chunk_id": chunk["id"],
                        "vector": embedding_list,
                        "error": None
                    })
                
                if (i + batch_size) % 100 == 0 or (i + batch_size) >= len(chunks):
                    eprint(f"[embed] Processed {min(i + batch_size, len(chunks))}/{len(chunks)} chunks...")
                    
            except Exception as e:
                eprint(f"[embed] ERROR: Failed to process batch starting at {i}: {e}")
                # Add error entries for failed batch
                for chunk in batch:
                    results.append({
                        "chunk_id": chunk["id"],
                        "vector": [],
                        "error": str(e)
                    })
        
        eprint(f"[embed] ✓ Successfully generated {len([r for r in results if r['error'] is None])} embeddings")
        
    except Exception as e:
        eprint(f"[embed] ERROR: Unexpected error during embedding generation: {e}")
        sys.exit(1)

    # Output results as JSON to stdout
    try:
        print(json.dumps(results))
    except Exception as e:
        eprint(f"[embed] ERROR: Failed to serialize output: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
