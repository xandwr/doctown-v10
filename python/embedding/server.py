# server.py
from fastapi import FastAPI
from pydantic import BaseModel
import uvicorn
from model import EmbeddingModel

app = FastAPI()
model = EmbeddingModel()

class EmbedRequest(BaseModel):
    texts: list[str]

@app.post("/embed")
def embed(req: EmbedRequest):
    vectors = model.embed(req.texts)
    return {"embeddings": vectors.tolist()}

if __name__ == "__main__":
    uvicorn.run(app, host="0.0.0.0", port=18115)
