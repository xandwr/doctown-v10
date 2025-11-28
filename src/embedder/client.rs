// client.rs - the HTTP/Subprocess embedder
use crate::embedding::types::*;
use reqwest::Client;

pub struct EmbeddingClient {
    http: Client,
    endpoint: String,
}

impl EmbeddingClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self {
            http: Client::new(),
            endpoint: endpoint.into(),
        }
    }

    pub async fn embed(&self, texts: Vec<String>) -> anyhow::Result<Vec<Vec<f32>>> {
        let req = EmbeddingRequest { texts };
        let res: EmbeddingResponse = self
            .http
            .post(format!("{}/embed", self.endpoint))
            .json(&req)
            .send()
            .await?
            .json()
            .await?;

        Ok(res.embeddings)
    }
}
