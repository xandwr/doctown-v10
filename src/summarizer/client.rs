use reqwest::Client;
use std::time::Duration;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SummarizerError {
    #[error("HTTP request failed: {0}")]
    RequestFailed(#[from] reqwest::Error),

    #[error("Server returned error status {status}: {body}")]
    ServerError { status: u16, body: String },

    #[error("Timeout after {0:?}")]
    Timeout(Duration),
}

pub struct DocumenterClient {
    http: Client,
    endpoint: String,
    #[allow(dead_code)]
    timeout: Duration,
}

impl DocumenterClient {
    pub fn new(endpoint: impl Into<String>) -> Self {
        Self::with_timeout(endpoint, Duration::from_secs(180)) // 3 min for LLM generation
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

    /// Check if the summarizer server is healthy
    pub async fn health_check(&self) -> Result<HealthResponse, SummarizerError> {
        let response = self
            .http
            .get(format!("{}/health", self.endpoint))
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SummarizerError::ServerError {
                status: status.as_u16(),
                body,
            });
        }

        Ok(response.json().await?)
    }

    /// Summarize a text/code chunk
    pub async fn summarize(
        &self,
        text: String,
        instructions: Option<String>,
    ) -> Result<String, SummarizerError> {
        self.summarize_with_system(text, instructions, None).await
    }

    /// Summarize with optional system prompt override
    pub async fn summarize_with_system(
        &self,
        text: String,
        instructions: Option<String>,
        system_prompt: Option<String>,
    ) -> Result<String, SummarizerError> {
        let req = SummarizeRequest {
            text,
            instructions,
            system_prompt,
        };

        let response = self
            .http
            .post(format!("{}/summarize", self.endpoint))
            .json(&req)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            let body = response
                .text()
                .await
                .unwrap_or_else(|_| "Unknown error".to_string());
            return Err(SummarizerError::ServerError {
                status: status.as_u16(),
                body,
            });
        }

        let res: SummarizeResponse = response.json().await?;
        Ok(res.summary)
    }
}

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub model: String,
    pub available_models: Vec<String>,
}
