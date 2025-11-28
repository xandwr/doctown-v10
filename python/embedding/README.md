# Python - embedding/

This is the brains of the embedding portion, outsourcing the heavy lifting to the tried and true `sentence_transformers`.
- Be a GPU-backed vector furnace
- Accept text -> spit out normalized floats
- Stay alive as a microservice during the whole docgen job
- Make embedding fast, predictable, and model-agnostic

## Contains:
- model.py
    - Loads SentenceTransformer
    - Decides GPU vs CPU
    - Provides a clean method:
        - embed(texts: list[str]) -> list[list[float]]
    - Does all heavy lifting (matrix ops, batching inside the model)

- server.py
    - FastAPI or simple Flask app
    - Loads the model once at startup
    - Exposes POST /embed
    - Receives { "texts": [...strings...] }
    - Responds { "embeddings": [...vectors...] }
    - Keeps the model warm across hundreds/thousands of calls (this is huge for latency and throughput)

- client.py
    - CLI/subprocess version if Rust doesn’t want HTTP
    - Reads JSON from stdin
    - Writes embeddings to stdout