pub mod chat;
pub mod embed;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Role {
    System,
    User,
    Assistant,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Source {
    pub idx: usize,
    pub chunk_id: String,
    pub page: u32,
    pub snippet: String,
    pub quads: Vec<Quad>,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub struct Quad {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
}

#[derive(Clone, Debug)]
pub struct ChatRequest {
    pub model: String,
    pub messages: Vec<ChatMessage>,
    pub system_prompt: String,
    pub temperature: Option<f32>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", content = "data", rename_all = "camelCase")]
pub enum ChatEvent {
    Sources(Vec<Source>),
    Token(String),
    Done,
    Error(String),
}

#[derive(Debug, thiserror::Error)]
pub enum LlmError {
    #[error("authentication failed — check your API key in Settings")]
    Auth,
    #[error("rate limited; retry shortly")]
    RateLimited,
    #[error("network error: {0}")]
    Network(String),
    #[error("provider error: {0}")]
    Provider(String),
}

#[async_trait::async_trait]
pub trait LlmProvider: Send + Sync {
    fn id(&self) -> &'static str;
    async fn chat_stream(
        &self,
        req: ChatRequest,
    ) -> Result<futures_util::stream::BoxStream<'static, Result<String, LlmError>>, LlmError>;

    async fn chat_once(&self, req: ChatRequest) -> Result<String, LlmError> {
        log::debug!("chat_once request: {:?}", req);
        let mut stream = self.chat_stream(req).await?;
        let mut out = String::new();
        while let Some(tok) = stream.next().await {
            out.push_str(&tok?);
        }
        log::debug!("chat_once response: {}", out);
        Ok(out)
    }
}

#[derive(Clone, Copy, Debug)]
pub enum EmbedInput {
    Document,
    Query,
}

#[async_trait::async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn id(&self) -> String;
    fn dimensions(&self) -> usize;
    async fn embed(&self, texts: &[String], kind: EmbedInput) -> Result<Vec<Vec<f32>>, LlmError>;
}
