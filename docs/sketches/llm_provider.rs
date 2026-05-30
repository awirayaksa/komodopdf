//! DESIGN SKETCH — not wired up. Provider-agnostic LLM/embedding abstraction.
//!
//! Goal (SPEC §3): define the interface ONCE, implement ONE provider (cloud BYO-key)
//! for v0.1, then add Ollama (v0.2) and more (v0.3) behind the same trait — no churn
//! in the rest of the app.
//!
//! Lives on the Rust/Tauri side so API keys and network calls never touch the webview.
//! Keys are read from the OS keychain, never logged, never sent anywhere but the
//! provider the user chose.

use async_trait::async_trait;
use futures::stream::BoxStream;
use serde::{Deserialize, Serialize};

// ---------- Shared types ----------

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: Role,
    pub content: String,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Role {
    System,
    User,
    Assistant,
}

/// A retrieved source the model was grounded on, surfaced back to the UI so the
/// chat can render clickable citations. `page` + `quads` let the reader scroll to
/// and flash-highlight the exact span (SPEC §9.1 — the magic moment).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Citation {
    pub chunk_id: String,
    pub page: u32,          // 1-based page index in the PDF
    pub snippet: String,    // short preview shown in the citation chip
    pub quads: Vec<Quad>,   // on-page rectangles to highlight (PDF user-space coords)
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
    /// Already-assembled grounding context (retrieved chunks + forced current
    /// page/selection). The RAG layer builds this; the provider just sends it.
    pub system_prompt: String,
    pub temperature: Option<f32>, // hidden/advanced — default None (SPEC §6)
}

/// A streamed chunk of the assistant's reply. Citations arrive once, up front,
/// because the RAG layer knows them before generation starts.
#[derive(Clone, Debug)]
pub enum ChatEvent {
    Citations(Vec<Citation>),
    Token(String),
    Done,
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

// ---------- The interface (implement once, swap freely) ----------

#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// Stable id, e.g. "anthropic", "openai", "ollama".
    fn id(&self) -> &'static str;

    /// Models the user can pick in the panel (SPEC §6).
    async fn list_models(&self) -> Result<Vec<String>, LlmError>;

    /// Streaming chat completion. The RAG layer has already assembled grounding
    /// into `req.system_prompt` and knows the citations.
    async fn chat_stream(
        &self,
        req: ChatRequest,
        citations: Vec<Citation>,
    ) -> Result<BoxStream<'static, Result<ChatEvent, LlmError>>, LlmError>;
}

/// Embeddings are a separate concern: a provider may do chat but delegate
/// embeddings to a local model (common in local mode — local embeddings,
/// cloud chat, or vice versa). Keep them decoupled.
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn dimensions(&self) -> usize;
    /// Batch for throughput; callers chunk large books.
    async fn embed(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, LlmError>;
}

// ---------- Registry ----------

/// Resolves the user's configured provider at call time. v0.1 ships exactly one
/// chat provider; the registry exists so v0.2+ additions are config, not surgery.
pub struct ProviderRegistry {
    pub chat: Box<dyn LlmProvider>,
    pub embeddings: Box<dyn EmbeddingProvider>,
}

// v0.1: AnthropicProvider (or OpenAiProvider) implements LlmProvider + an
//       EmbeddingProvider (or reuse a local embedding model from day one to keep
//       the index portable across chat providers — recommended).
// v0.2: OllamaProvider implements both, becomes the default.
