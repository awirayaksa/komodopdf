use crate::llm::{ChatRequest, LlmError, LlmProvider, Role};
use async_trait::async_trait;
use futures_util::stream::{BoxStream, StreamExt};
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use serde::{Deserialize, Serialize};

pub struct OpenAiCompatibleProvider {
    client: reqwest::Client,
    base_url: String,
    api_key: String,
    model: String,
}

impl OpenAiCompatibleProvider {
    pub fn new(base_url: String, api_key: String, model: String) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.trim_end_matches('/').to_string(),
            api_key,
            model,
        }
    }
}

#[derive(Serialize)]
struct OpenAiChatRequest {
    model: String,
    messages: Vec<OpenAiMessage>,
    stream: bool,
    temperature: Option<f32>,
}

#[derive(Serialize)]
struct OpenAiMessage {
    role: String,
    content: String,
}

#[derive(Deserialize, Debug)]
struct OpenAiStreamChunk {
    choices: Vec<OpenAiChoice>,
}

#[derive(Deserialize, Debug)]
struct OpenAiChoice {
    delta: OpenAiDelta,
}

#[derive(Deserialize, Debug)]
struct OpenAiDelta {
    content: Option<String>,
}

#[async_trait]
impl LlmProvider for OpenAiCompatibleProvider {
    fn id(&self) -> &'static str {
        "openai-compatible"
    }

    async fn chat_stream(
        &self,
        req: ChatRequest,
    ) -> Result<BoxStream<'static, Result<String, LlmError>>, LlmError> {
        let url = format!("{}/chat/completions", self.base_url);

        log::debug!("LLM system_prompt: {}", req.system_prompt);
        for (i, m) in req.messages.iter().enumerate() {
            log::debug!("LLM message[{}] role={:?} content={}", i, m.role, m.content);
        }

        let messages: Vec<OpenAiMessage> = std::iter::once(OpenAiMessage {
            role: "system".to_string(),
            content: req.system_prompt,
        })
        .chain(req.messages.into_iter().map(|m| OpenAiMessage {
            role: match m.role {
                Role::System => "system".to_string(),
                Role::User => "user".to_string(),
                Role::Assistant => "assistant".to_string(),
            },
            content: m.content,
        }))
        .collect();

        let body = OpenAiChatRequest {
            model: self.model.clone(),
            messages,
            stream: true,
            temperature: req.temperature,
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

        log::debug!(
            "LLM request to {}: {}",
            url,
            serde_json::to_string_pretty(&body)
                .unwrap_or_else(|e| format!("(serialization error: {})", e))
        );

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

        let mut buffer: Vec<u8> = Vec::new();
        let byte_stream = response.bytes_stream();
        let stream = byte_stream.map(move |chunk| {
            let chunk = chunk.map_err(|e| LlmError::Network(e.to_string()))?;
            buffer.extend_from_slice(&chunk);

            let mut tokens = Vec::new();
            while let Some(pos) = buffer.iter().position(|&b| b == b'\n') {
                let line_bytes: Vec<u8> = buffer.drain(..=pos).collect();
                let line_bytes = if line_bytes.ends_with(b"\r\n") {
                    &line_bytes[..line_bytes.len() - 2]
                } else {
                    &line_bytes[..line_bytes.len() - 1]
                };
                let line = String::from_utf8_lossy(line_bytes);
                let line = line.strip_prefix("data: ").unwrap_or(&line);
                if line.trim() == "[DONE]" || line.trim().is_empty() {
                    continue;
                }
                let parsed: Result<OpenAiStreamChunk, _> = serde_json::from_str(line);
                if let Ok(data) = parsed {
                    for choice in data.choices {
                        if let Some(content) = choice.delta.content {
                            tokens.push(content);
                        }
                    }
                }
            }
            log::debug!("LLM response tokens: {:?}", tokens);
            Ok(tokens)
        });

        let flat_stream = stream.flat_map(|result| {
            let items: Vec<Result<String, LlmError>> = match result {
                Ok(tokens) => tokens.into_iter().map(Ok).collect(),
                Err(e) => vec![Err(e)],
            };
            futures_util::stream::iter(items)
        });

        Ok(Box::pin(flat_stream))
    }
}
