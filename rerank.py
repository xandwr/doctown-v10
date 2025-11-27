#!/usr/bin/env python3
"""
Reranking script for doctown.
Reads query and chunks from stdin as JSON, reranks them, outputs scores to stdout as JSON.
"""

import sys
import json
import argparse
from typing import List, Dict, Any

def eprint(*args, **kwargs):
    """Print to stderr for logging"""
    print(*args, file=sys.stderr, **kwargs)

def main():
    parser = argparse.ArgumentParser(description="Rerank code chunks based on query")
    parser.add_argument("--model", required=True, help="Model name to use for reranking")
    args = parser.parse_args()

    eprint(f"[rerank] Loading model: {args.model}")

    try:
        # Import heavy dependencies only after arg parsing
        from sentence_transformers import CrossEncoder
        
        # Load reranker model
        model = CrossEncoder(args.model)
        eprint(f"[rerank] ✓ Model loaded successfully")
        
    except Exception as e:
        eprint(f"[rerank] ERROR: Failed to load model: {e}")
        sys.exit(1)

    # Read input from stdin
    try:
        input_data = sys.stdin.read()
        items = json.loads(input_data)
        eprint(f"[rerank] Processing {len(items)} items...")
    except json.JSONDecodeError as e:
        eprint(f"[rerank] ERROR: Failed to parse input JSON: {e}")
        sys.exit(1)
    except Exception as e:
        eprint(f"[rerank] ERROR: Failed to read input: {e}")
        sys.exit(1)

    # Prepare pairs for reranking
    results = []
    batch_size = 32
    
    try:
        for i in range(0, len(items), batch_size):
            batch = items[i:i + batch_size]
            
            try:
                # Create query-document pairs
                pairs = [[item["query"], item["content"]] for item in batch]
                
                # Get scores
                scores = model.predict(pairs, show_progress_bar=False)
                
                # Convert to results
                for j, item in enumerate(batch):
                    results.append({
                        "chunk_id": item["chunk_id"],
                        "score": float(scores[j]),
                        "error": None
                    })
                
                if (i + batch_size) % 100 == 0 or (i + batch_size) >= len(items):
                    eprint(f"[rerank] Processed {min(i + batch_size, len(items))}/{len(items)} items...")
                    
            except Exception as e:
                eprint(f"[rerank] ERROR: Failed to process batch starting at {i}: {e}")
                # Add error entries for failed batch
                for item in batch:
                    results.append({
                        "chunk_id": item["chunk_id"],
                        "score": 0.0,
                        "error": str(e)
                    })
        
        eprint(f"[rerank] ✓ Successfully reranked {len([r for r in results if r['error'] is None])} items")
        
    except Exception as e:
        eprint(f"[rerank] ERROR: Unexpected error during reranking: {e}")
        sys.exit(1)

    # Output results as JSON to stdout
    try:
        print(json.dumps(results))
    except Exception as e:
        eprint(f"[rerank] ERROR: Failed to serialize output: {e}")
        sys.exit(1)

if __name__ == "__main__":
    main()
