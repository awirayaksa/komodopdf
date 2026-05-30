use crate::llm::{EmbedInput, EmbeddingProvider, LlmError};
use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

pub struct OpenAiCompatibleEmbedder {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
    dimensions: usize,
}

impl OpenAiCompatibleEmbedder {
    pub fn new(base_url: String, api_key: String, model: String, dimensions: usize) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
            dimensions,
        }
    }
}

#[derive(Serialize)]
struct EmbedRequest {
    model: String,
    input: Vec<String>,
}

#[derive(Deserialize)]
struct EmbedResponse {
    data: Vec<EmbedData>,
}

#[derive(Deserialize)]
struct EmbedData {
    embedding: Vec<f32>,
}

#[async_trait]
impl EmbeddingProvider for OpenAiCompatibleEmbedder {
    fn id(&self) -> String {
        format!("remote:{}/{}", self.base_url, self.model)
    }

    fn dimensions(&self) -> usize {
        self.dimensions
    }

    async fn embed(&self, texts: &[String], _kind: EmbedInput) -> Result<Vec<Vec<f32>>, LlmError> {
        let url = format!("{}/embeddings", self.base_url);
        let body = EmbedRequest {
            model: self.model.clone(),
            input: texts.to_vec(),
        };

        let mut headers = HeaderMap::new();
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        if !self.api_key.is_empty() {
            let auth = format!("Bearer {}", self.api_key);
            headers.insert(
                AUTHORIZATION,
                HeaderValue::from_str(&auth).map_err(|e| LlmError::Network(e.to_string()))?,
            );
        }

        let response = self
            .client
            .post(&url)
            .headers(headers)
            .json(&body)
            .send()
            .await
            .map_err(|e| {
                let status = e.status();
                if status == Some(reqwest::StatusCode::UNAUTHORIZED)
                    || status == Some(reqwest::StatusCode::FORBIDDEN)
                {
                    LlmError::Auth
                } else if status == Some(reqwest::StatusCode::TOO_MANY_REQUESTS) {
                    LlmError::RateLimited
                } else if e.is_connect() || e.is_timeout() {
                    LlmError::Network(e.to_string())
                } else {
                    LlmError::Network(e.to_string())
                }
            })?;

        let status = response.status();
        if !status.is_success() {
            let text = response.text().await.unwrap_or_default();
            if status == reqwest::StatusCode::UNAUTHORIZED || status == reqwest::StatusCode::FORBIDDEN {
                return Err(LlmError::Auth);
            } else if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                return Err(LlmError::RateLimited);
            } else {
                return Err(LlmError::Provider(format!("HTTP {}: {}", status, text)));
            }
        }

        let result: EmbedResponse = response
            .json()
            .await
            .map_err(|e| LlmError::Network(e.to_string()))?;
        Ok(result.data.into_iter().map(|d| d.embedding).collect())
    }
}
