# client.py
import sys
import json
from model import EmbeddingModel

model = EmbeddingModel()

def main():
    stdin = sys.stdin.read()
    data = json.loads(stdin)
    texts = data["texts"]
    vectors = model.embed(texts)

    out = {"embeddings": vectors.tolist()}
    print(json.dumps(out))

if __name__ == "__main__":
    main()
