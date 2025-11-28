"""
FastAPI server for local LLM-based code summarization.

Provides HTTP endpoints for summarizing code chunks using
local language models with zero cloud dependencies.
"""

import os
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel, Field
import uvicorn
from model import DocumenterModel
from registry import list_available_models, get_model_info

# Get model from environment or use default
MODEL_NAME = os.getenv("DOCUMENTER_MODEL", "qwen3-1.7b")
PORT = int(os.getenv("DOCUMENTER_PORT", "18116"))

app = FastAPI(
    title="Doctown Summarizer",
    description="Local LLM-based code summarization service",
    version="1.0.0"
)

# Global model instance (loaded on startup)
model: DocumenterModel | None = None


@app.on_event("startup")
async def load_model():
    """Load the model on server startup."""
    global model
    print(f"[server] Starting documenter server on port {PORT}")
    print(f"[server] Requested model: {MODEL_NAME}")

    try:
        model = DocumenterModel(model_name=MODEL_NAME)
        print("[server] Model loaded and ready")
    except Exception as e:
        print(f"[server] ERROR: Failed to load model: {e}")
        raise


class SummarizeRequest(BaseModel):
    """Request for summarizing text/code."""
    text: str = Field(..., description="The code or text to summarize")
    instructions: str | None = Field(None, description="Optional custom instructions")
    system_prompt: str | None = Field(None, description="Optional system prompt override")


class SummarizeResponse(BaseModel):
    """Response containing the generated summary."""
    summary: str = Field(..., description="The generated summary")


class HealthResponse(BaseModel):
    """Health check response."""
    status: str
    model: str
    available_models: list[str]


@app.get("/health", response_model=HealthResponse)
async def health_check():
    """
    Health check endpoint.

    Returns server status and loaded model information.
    """
    return HealthResponse(
        status="healthy" if model is not None else "model_not_loaded",
        model=MODEL_NAME,
        available_models=list_available_models()
    )


@app.get("/models")
async def list_models():
    """
    List all available models in the registry.

    Returns model names and descriptions.
    """
    models = {}
    for model_name in list_available_models():
        try:
            info = get_model_info(model_name)
            models[model_name] = info
        except Exception as e:
            models[model_name] = f"Error: {e}"

    return {"models": models}


@app.post("/summarize", response_model=SummarizeResponse)
async def summarize(req: SummarizeRequest):
    """
    Generate a summary of the provided text/code.

    Args:
        req: SummarizeRequest with text and optional instructions

    Returns:
        SummarizeResponse with the generated summary

    Raises:
        HTTPException: If model is not loaded or generation fails
    """
    if model is None:
        raise HTTPException(
            status_code=503,
            detail="Model not loaded. Server may still be initializing."
        )

    if not req.text or not req.text.strip():
        raise HTTPException(
            status_code=400,
            detail="Text field cannot be empty"
        )

    try:
        summary = model.summarize(
            text=req.text,
            instructions=req.instructions or "",
            system_prompt=req.system_prompt or ""
        )

        return SummarizeResponse(summary=summary)

    except Exception as e:
        print(f"[server] Error during summarization: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Summarization failed: {str(e)}"
        )


if __name__ == "__main__":
    print(f"[server] Available models: {', '.join(list_available_models())}")
    print(f"[server] To use a different model, set DOCUMENTER_MODEL environment variable")

    uvicorn.run(
        app,
        host="0.0.0.0",
        port=PORT,
        log_level="info"
    )
