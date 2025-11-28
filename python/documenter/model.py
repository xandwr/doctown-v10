"""
Local LLM model wrapper for code summarization.

Uses the model registry to load and configure different LLMs
with appropriate parameters and settings.
"""

from transformers import AutoModelForCausalLM, AutoTokenizer, BitsAndBytesConfig
import torch
from registry import get_model_config, list_available_models


class DocumenterModel:
    """
    Wrapper for local LLM models used for code summarization.

    Uses registry-based configuration to support multiple models
    with different parameters and capabilities.
    """

    def __init__(self, model_name: str = "qwen3-1.7b"):
        """
        Initialize the model from registry.

        Args:
            model_name: Name of model in registry (e.g., "qwen3-1.7b")
        """
        self.config = get_model_config(model_name)
        self.model_name = model_name

        device = "cuda" if torch.cuda.is_available() else "cpu"
        print(f"[documenter] Loading {model_name} ({self.config.model_id}) on {device}...")
        print(f"[documenter] {self.config.description}")

        # Load tokenizer
        self.tokenizer = AutoTokenizer.from_pretrained(self.config.model_id)

        # Ensure pad token is set
        if self.tokenizer.pad_token is None:
            self.tokenizer.pad_token = self.tokenizer.eos_token

        # Build model loading kwargs
        model_kwargs = {
            "torch_dtype": torch.float16 if device == "cuda" else torch.float32,
            "device_map": "auto" if device == "cuda" else None,
        }

        # Apply quantization if configured
        if self.config.load_in_8bit:
            print("[documenter] Loading in 8-bit mode for memory efficiency")
            model_kwargs["load_in_8bit"] = True
            model_kwargs["quantization_config"] = BitsAndBytesConfig(load_in_8bit=True)
        elif self.config.load_in_4bit:
            print("[documenter] Loading in 4-bit mode for memory efficiency")
            model_kwargs["load_in_4bit"] = True
            model_kwargs["quantization_config"] = BitsAndBytesConfig(load_in_4bit=True)

        # Load model
        self.model = AutoModelForCausalLM.from_pretrained(
            self.config.model_id,
            **model_kwargs
        )

        self.model.eval()
        print(f"[documenter] Model loaded successfully")

    def _format_prompt(self, user_message: str, system_message: str = "") -> str:
        """
        Format the prompt according to model's chat template.

        Args:
            user_message: The main prompt/instruction
            system_message: Optional system prompt

        Returns:
            Formatted prompt string
        """
        if self.config.requires_chat_template:
            messages = []

            if system_message and self.config.supports_system_prompt:
                messages.append({"role": "system", "content": system_message})

            messages.append({"role": "user", "content": user_message})

            # Use tokenizer's chat template
            return self.tokenizer.apply_chat_template(
                messages,
                tokenize=False,
                add_generation_prompt=True
            )
        else:
            # Simple concatenation for models without chat template
            if system_message:
                return f"{system_message}\n\n{user_message}"
            return user_message

    def summarize(self, text: str, instructions: str = "", system_prompt: str = "") -> str:
        """
        Generate a summary of the given text.

        Args:
            text: The text/code to summarize
            instructions: Specific instructions for this summary
            system_prompt: Optional system-level prompt

        Returns:
            Generated summary as a string
        """
        # Build the user message
        if instructions:
            user_message = f"{instructions}\n\n{text}"
        else:
            user_message = f"Summarize the following code:\n\n{text}"

        # Format according to model's requirements
        prompt = self._format_prompt(user_message, system_prompt)

        # Tokenize
        inputs = self.tokenizer(
            prompt,
            return_tensors="pt",
            truncation=True,
            max_length=4096  # Most models support at least 4k context
        ).to(self.model.device)

        # Generate with model-specific parameters
        with torch.no_grad():
            outputs = self.model.generate(
                **inputs,
                max_new_tokens=self.config.max_new_tokens,
                temperature=self.config.temperature,
                do_sample=self.config.do_sample,
                top_p=self.config.top_p,
                top_k=self.config.top_k,
                repetition_penalty=self.config.repetition_penalty,
                eos_token_id=self.config.eos_token_id or self.tokenizer.eos_token_id,
                pad_token_id=self.config.pad_token_id or self.tokenizer.pad_token_id,
            )

        # Decode output
        full_output = self.tokenizer.decode(outputs[0], skip_special_tokens=True)

        # Extract just the generated part (remove the prompt)
        # For chat models, the response comes after the prompt
        if self.config.requires_chat_template:
            # Try to extract assistant response
            if "assistant" in full_output.lower():
                parts = full_output.split("assistant", 1)
                if len(parts) > 1:
                    return parts[1].strip().lstrip(":").strip()

        # Fallback: remove the original prompt from output
        if prompt in full_output:
            return full_output.replace(prompt, "").strip()

        return full_output.strip()
