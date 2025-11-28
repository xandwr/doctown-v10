// client.rs - the HTTP/Subprocess embedder
use crate::embedder::types::*;
use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum EmbedError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Server returned error status {status}: {body}")]
    ServerError { status: u16, body: String },

    #[error("Invalid response format: {0}")]
    InvalidResponse(String),

    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}

pub struct EmbeddingClient {
    http: Client,
    endpoint: String,
    #[allow(dead_code)]
    timeout: Duration,
}

impl EmbeddingClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self::with_timeout(endpoint, Duration::from_secs(120))
    }

    pub fn with_timeout(endpoint: impl Into<String>, timeout: Duration) -> Self {
        let http = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            http,
            endpoint: endpoint.into(),
            timeout,
        }
    }

    pub async fn embed(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, EmbedError> {
        if texts.is_empty() {
            return Ok(Vec::new());
        }

        let req = EmbeddingRequest { texts };
        let response = self
            .http
            .post(format!("{}/embed", self.endpoint))
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(EmbedError::ServerError {
                status: status.as_u16(),
                body,
            });
        }

        let res: EmbeddingResponse = response.json().await?;

        Ok(res.embeddings)
    }

    pub async fn embed_chunks(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>, EmbedError> {
        self.embed(texts).await
    }
}
