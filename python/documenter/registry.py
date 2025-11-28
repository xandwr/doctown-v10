"""
Model registry for local LLMs.

Each model has specific configuration for generation parameters,
chat template handling, and special features like chain-of-thought.
"""

from dataclasses import dataclass
from typing import Optional


@dataclass
class ModelConfig:
    """Configuration for a specific LLM model."""

    # HuggingFace model identifier
    model_id: str

    # Generation parameters
    max_new_tokens: int = 512
    temperature: float = 0.3
    do_sample: bool = True
    top_p: float = 0.9
    top_k: int = 50
    repetition_penalty: float = 1.1

    # Model-specific features
    supports_system_prompt: bool = True
    has_thinking_mode: bool = False
    requires_chat_template: bool = True

    # Token management
    eos_token_id: Optional[int] = None
    pad_token_id: Optional[int] = None

    # Memory optimization
    use_flash_attention: bool = False
    load_in_8bit: bool = False
    load_in_4bit: bool = False

    # Description
    description: str = ""


# Model Registry
MODEL_REGISTRY = {
    "qwen3-1.7b": ModelConfig(
        model_id="Qwen/Qwen2.5-1.5B-Instruct",
        max_new_tokens=512,
        temperature=0.3,
        do_sample=True,
        top_p=0.9,
        supports_system_prompt=True,
        has_thinking_mode=False,
        requires_chat_template=True,
        use_flash_attention=False,
        description="Qwen 2.5 1.5B - Fast, efficient model for code summarization"
    ),

    "qwen3-3b": ModelConfig(
        model_id="Qwen/Qwen2.5-3B-Instruct",
        max_new_tokens=512,
        temperature=0.3,
        do_sample=True,
        top_p=0.9,
        supports_system_prompt=True,
        has_thinking_mode=False,
        requires_chat_template=True,
        description="Qwen 2.5 3B - Better quality, still fast"
    ),

    "qwen3-7b": ModelConfig(
        model_id="Qwen/Qwen2.5-7B-Instruct",
        max_new_tokens=512,
        temperature=0.3,
        do_sample=True,
        top_p=0.9,
        supports_system_prompt=True,
        has_thinking_mode=False,
        requires_chat_template=True,
        load_in_8bit=True,  # Use quantization for 7B
        description="Qwen 2.5 7B - High quality, requires more VRAM"
    ),

    "phi-3-mini": ModelConfig(
        model_id="microsoft/Phi-3-mini-4k-instruct",
        max_new_tokens=512,
        temperature=0.3,
        do_sample=True,
        supports_system_prompt=True,
        has_thinking_mode=False,
        requires_chat_template=True,
        description="Phi-3 Mini - Microsoft's efficient 3.8B model"
    ),

    "deepseek-coder-1.3b": ModelConfig(
        model_id="deepseek-ai/deepseek-coder-1.3b-instruct",
        max_new_tokens=512,
        temperature=0.3,
        do_sample=True,
        supports_system_prompt=False,  # Uses special format
        has_thinking_mode=False,
        requires_chat_template=False,
        description="DeepSeek Coder 1.3B - Code-specialized model"
    ),
}


def get_model_config(model_name: str) -> ModelConfig:
    """
    Get configuration for a registered model.

    Args:
        model_name: Name of the model in the registry

    Returns:
        ModelConfig for the specified model

    Raises:
        ValueError: If model is not in registry
    """
    if model_name not in MODEL_REGISTRY:
        available = ", ".join(MODEL_REGISTRY.keys())
        raise ValueError(
            f"Model '{model_name}' not found in registry. "
            f"Available models: {available}"
        )

    return MODEL_REGISTRY[model_name]


def list_available_models() -> list[str]:
    """List all available model names in the registry."""
    return list(MODEL_REGISTRY.keys())


def get_model_info(model_name: str) -> str:
    """Get human-readable information about a model."""
    config = get_model_config(model_name)
    info = [
        f"Model: {model_name}",
        f"ID: {config.model_id}",
        f"Description: {config.description}",
        f"Max tokens: {config.max_new_tokens}",
        f"Temperature: {config.temperature}",
    ]

    if config.has_thinking_mode:
        info.append("⚠ Has thinking mode enabled")

    if config.load_in_8bit:
        info.append("📦 Loads in 8-bit mode")
    elif config.load_in_4bit:
        info.append("📦 Loads in 4-bit mode")

    return "\n".join(info)
