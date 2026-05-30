use crate::llm::{ChatMessage, ChatRequest, LlmProvider, Role};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub struct Verdict {
    pub ok: bool,
    pub reason: String,
}

pub async fn validate_answer(
    provider: &dyn LlmProvider,
    model: &str,
    question: &str,
    context: &str,
    answer: &str,
) -> Verdict {
    let system = "You are a strict answer evaluator. \
Evaluate whether the following answer is coherent, sensible, and addresses the user's question. \
The answer was generated using retrieved document context. \
Return ONLY a JSON object with no markdown, no code fences, and no extra text. \
The JSON must have exactly two fields: \"ok\" (boolean) and \"reason\" (short string). \
Set ok=true if the answer is clear, coherent, and relevant. \
Set ok=false if the answer is garbled, nonsensical, off-topic, empty, or does not address the question.";

    let user = format!(
        "User question: {}\n\nRetrieved context:\n{}\n\nDraft answer:\n{}\n\nReturn JSON verdict only.",
        question, context, answer
    );

    let req = ChatRequest {
        model: model.to_string(),
        messages: vec![ChatMessage {
            role: Role::User,
            content: user,
        }],
        system_prompt: system.to_string(),
        temperature: Some(0.0),
    };

    match provider.chat_once(req).await {
        Ok(text) => {
            if let Some(start) = text.find('{') {
                if let Some(end) = text.rfind('}') {
                    let json = &text[start..=end];
                    if let Ok(verdict) = serde_json::from_str::<Verdict>(json) {
                        return verdict;
                    }
                }
            }
            eprintln!("Judge returned unparseable output: {}", text);
            Verdict {
                ok: true,
                reason: "judge output unparseable, fail-open".to_string(),
            }
        }
        Err(e) => {
            eprintln!("Judge call failed: {}", e);
            Verdict {
                ok: true,
                reason: format!("judge call failed: {}", e),
            }
        }
    }
}
