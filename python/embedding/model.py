# model.py
from sentence_transformers import SentenceTransformer
import torch

class EmbeddingModel:
    def __init__(self, model_name="google/embeddinggemma-300m"):
        device = "cuda" if torch.cuda.is_available() else "cpu"
        print(f"[embedding] Loading model {model_name} on {device}...")
        self.model = SentenceTransformer(model_name, device=device)

    def embed(self, texts, batch_size=32):
        """
        Embed a list of strings -> returns list of vectors
        """
        return self.model.encode(
            texts,
            batch_size=batch_size,
            normalize_embeddings=True,
            convert_to_numpy=True,
            show_progress_bar=False,
        )
