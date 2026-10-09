use crate::db::Db;
use crate::llm::{EmbedInput, EmbeddingProvider, LlmError, Quad, Source};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use tauri::ipc::Channel;

// validate module retired — replaced by confidence routing in lib.rs::ask

// Invalidate old indexes whose highlights used bottom-left PDF coordinates.
pub fn index_id(provider_id: &str) -> String { format!("{provider_id}:viewport-v2") }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageExtraction {
    pub page: u32,
    pub spans: Vec<TextSpan>,
    pub has_text: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TextSpan {
    pub text: String,
    pub quads: Vec<Quad>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Chunk {
    pub id: String,
    pub page: u32,
    pub text: String,
    pub quads: Vec<Quad>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IndexProgress {
    pub done: usize,
    pub total: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReaderContext {
    pub current_page: u32,
    pub selection_text: Option<String>,
    pub selection_quads: Vec<Quad>,
}

pub struct Grounding {
    pub system_prompt: String,
    pub knowledge_system_prompt: String,
    pub user_prompt: String,
    pub sources: Vec<Source>,
    pub max_score: f32,
}

fn approximate_tokens(text: &str) -> usize {
    text.chars().count() / 4
}

/// Split text into sentences more robustly than `split('.')`, avoiding
/// breaks inside decimals ("v1.5"), abbreviations ("e.g."), and code.
fn split_sentences(text: &str) -> Vec<String> {
    let mut sentences = Vec::new();
    let mut current = String::new();

    for (i, c) in text.char_indices() {
        current.push(c);
        if c == '.' || c == '!' || c == '?' {
            let after = i + c.len_utf8();
            let is_boundary = if after >= text.len() {
                true
            } else {
                let mut j = after;
                while j < text.len() && text.as_bytes()[j].is_ascii_whitespace() {
                    j += 1;
                }
                if j >= text.len() {
                    true
                } else {
                    let next_char = text[j..].chars().next().unwrap();
                    next_char.is_uppercase() || next_char.is_ascii_digit()
                }
            };

            let mut is_abbrev = false;
            if is_boundary {
                let words: Vec<&str> = current.split_whitespace().collect();
                if let Some(last_word) = words.last() {
                    let clean = last_word.trim_matches(|c: char| !c.is_alphabetic()).to_lowercase();
                    let abbrev = ["e", "i", "mr", "mrs", "dr", "prof", "vs", "fig", "etc", "al", "no", "vol", "ch", "pg", "pp"];
                    if abbrev.contains(&clean.as_str()) {
                        is_abbrev = true;
                    }
                }
            }

            if is_boundary && !is_abbrev {
                let trimmed = current.trim().to_string();
                if !trimmed.is_empty() {
                    sentences.push(trimmed);
                }
                current.clear();
            }
        }
    }

    if !current.trim().is_empty() {
        sentences.push(current.trim().to_string());
    }

    sentences
}

pub fn chunk_pages(pages: &[PageExtraction], target_tokens: usize) -> Vec<Chunk> {
    log::debug!("chunk_pages start: {} pages, target_tokens={}", pages.len(), target_tokens);
    let overlap_tokens = (target_tokens as f32 * 0.15) as usize;
    let mut chunks = Vec::new();

    for page in pages {
        if !page.has_text || page.spans.is_empty() {
            continue;
        }

        let full_text: String = page.spans.iter().map(|s| s.text.as_str()).collect();
        let all_quads: Vec<Quad> = page.spans.iter().flat_map(|s| s.quads.clone()).collect();

        if approximate_tokens(&full_text) <= target_tokens {
            chunks.push(Chunk {
                id: format!("page:{}:0", page.page),
                page: page.page,
                text: full_text,
                quads: all_quads,
            });
            continue;
        }

        let sentences = split_sentences(&full_text);
        let mut current_text = String::new();
        let mut current_quads: Vec<Quad> = Vec::new();
        let mut ord = 0;

        for sentence in &sentences {
            if sentence.is_empty() {
                continue;
            }
            current_text.push_str(sentence);
            current_text.push(' ');

            // Gather quads from any span whose text overlaps this sentence
            for span in &page.spans {
                let span_trim = span.text.trim();
                if span_trim.is_empty() {
                    continue;
                }
                // Simple overlap: the span contains part of the sentence or vice-versa
                if span_trim.contains(sentence) || sentence.contains(span_trim) {
                    current_quads.extend(span.quads.clone());
                }
            }

            if approximate_tokens(&current_text) >= target_tokens {
                // Deduplicate quads (stable sort + dedup)
                current_quads.sort_by(|a, b| {
                    a.x0.partial_cmp(&b.x0)
                        .unwrap_or(std::cmp::Ordering::Equal)
                        .then_with(|| a.y0.partial_cmp(&b.y0).unwrap_or(std::cmp::Ordering::Equal))
                });
                current_quads.dedup_by(|a, b| {
                    a.x0 == b.x0 && a.y0 == b.y0 && a.x1 == b.x1 && a.y1 == b.y1
                });

                chunks.push(Chunk {
                    id: format!("page:{}:{}", page.page, ord),
                    page: page.page,
                    text: current_text.trim().to_string(),
                    quads: current_quads,
                });
                ord += 1;

                // Overlap — char-safe slice
                let chars_to_keep = overlap_tokens * 4;
                let char_count = current_text.chars().count();
                if chars_to_keep > 0 && char_count > chars_to_keep {
                    let keep_start = current_text
                        .char_indices()
                        .rev()
                        .nth(chars_to_keep - 1)
                        .map(|(i, _)| i)
                        .unwrap_or(0);
                    current_text = current_text[keep_start..].to_string();
                } else {
                    current_text.clear();
                }
                current_quads = Vec::new();
            }
        }

        if !current_text.trim().is_empty() {
            current_quads.sort_by(|a, b| {
                a.x0.partial_cmp(&b.x0)
                    .unwrap_or(std::cmp::Ordering::Equal)
                    .then_with(|| a.y0.partial_cmp(&b.y0).unwrap_or(std::cmp::Ordering::Equal))
            });
            current_quads.dedup_by(|a, b| {
                a.x0 == b.x0 && a.y0 == b.y0 && a.x1 == b.x1 && a.y1 == b.y1
            });

            chunks.push(Chunk {
                id: format!("page:{}:{}", page.page, ord),
                page: page.page,
                text: current_text.trim().to_string(),
                quads: current_quads,
            });
        }
    }

    log::debug!("chunk_pages done: {} chunks produced", chunks.len());
    chunks
}

pub async fn index_document(
    db: &Db,
    file_hash: &str,
    pages: Vec<PageExtraction>,
    embedder: &dyn EmbeddingProvider,
    progress: Channel<IndexProgress>,
) -> Result<(), LlmError> {
    let total = pages.len();
    log::debug!("index_document start: file_hash={} pages={}", file_hash, total);

    progress.send(IndexProgress { done: 1, total }).ok();

    let chunks = chunk_pages(&pages, 700);
    if chunks.is_empty() { return Err(LlmError::Provider("No extractable text in this PDF".into())); }
    log::debug!("index_document chunked into {} chunks", chunks.len());

    progress.send(IndexProgress { done: 2, total }).ok();

    let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
    log::debug!("index_document embedding {} texts with kind=Document", texts.len());

    let embed_start = std::time::Instant::now();
    let embeddings = embedder.embed(&texts, EmbedInput::Document).await?;
    if embeddings.len() != chunks.len() { return Err(LlmError::Provider("Embedding provider returned an unexpected number of vectors".into())); }
    let embed_elapsed = embed_start.elapsed();
    log::info!(
        "index_document embed complete: {} vectors in {:?}",
        embeddings.len(),
        embed_elapsed
    );

    progress.send(IndexProgress { done: total.saturating_sub(1), total }).ok();

    let mut chunk_rows = Vec::new();
    for (i, (chunk, vector)) in chunks.iter().zip(embeddings.iter()).enumerate() {
        let quads_json = serde_json::to_string(&chunk.quads)
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        let mut vector_bytes = Vec::with_capacity(vector.len() * 4);
        for f in vector {
            vector_bytes.extend_from_slice(&f.to_le_bytes());
        }

        chunk_rows.push(crate::db::ChunkRow {
            id: format!("{}:{}:{}:{}", file_hash, chunk.page, chunk.id, chunk.text.len()),
            file_hash: file_hash.to_string(),
            page: chunk.page,
            text: chunk.text.clone(),
            quads_json,
            vector: vector_bytes,
        });

        if i % 50 == 0 {
            log::debug!("index_document preparing chunk_rows: {}/{}", i, chunks.len());
        }
    }
    log::debug!("index_document storing {} chunk rows", chunk_rows.len());

    db.store_chunks(file_hash, &chunk_rows)
        .map_err(|e| LlmError::Provider(e.to_string()))?;
    log::debug!("index_document store_chunks done");

    db.mark_indexed(file_hash, &index_id(&embedder.id()))
        .map_err(|e| LlmError::Provider(e.to_string()))?;
    log::debug!("index_document mark_indexed done");

    progress.send(IndexProgress { done: total, total }).ok();
    log::debug!("index_document finished successfully");

    Ok(())
}

fn rag_priority_system_prompt(persona: &str) -> String {
    format!("\
You are an AI reading assistant embedded in a PDF reader. \
You help readers understand this document.\n\n\
Persona: {}\n\n\
You have access to:\n\
1. The text of the page the reader is currently viewing (CURRENT SECTION — highest priority)\n\
2. Relevant excerpts retrieved from elsewhere in the document (RETRIEVED EXCERPTS)\n\
3. Your own general knowledge of the subject\n\n\
PRIORITY RULES — follow these strictly:\n\n\
- The document's content is your primary source of truth. If the document explains a concept \
in a particular way, that is the answer you give.\n\n\
- Use your general knowledge only to supplement, clarify, or simplify — never to override or \
contradict what the document says without explicitly flagging it.\n\n\
- If your general knowledge differs from the document's explanation, present the document's \
position first as the canonical answer, then optionally note the broader perspective with a \
phrase like: \"Note: more broadly in the field, this is sometimes described as...\"\n\n\
- If the question cannot be answered from the provided context at all, say so clearly, then \
answer from general knowledge if the topic is relevant to this document's domain.\n\n\
- Do not invent content not present in either the document or your verified knowledge. \
When uncertain, say so.\n\n\
BEHAVIOR:\n\n\
- Always anchor your answer to the document's content and indicate which page you are drawing \
from using [n] citation numbers that match the numbered excerpts in the context.\n\n\
- Simplify, use analogies, and adjust your explanation style freely — even if the document \
does not explain it that way — as long as the substance matches the document's position.\n\n\
- If the reader asks to explain something \"in simple terms\" or \"like a beginner,\" answer \
fully and accessibly while keeping it grounded in the document's framing.\n\n\
- Keep responses concise and conversational. Aim for clarity over completeness.\n\n\
FORMATTING RULES — follow these strictly:\n\n\
- Use proper Markdown syntax for emphasis: write `**bold**` and `*italic*` with no spaces between the asterisks and the text.\n\
- Never start bold or italic on one line and close it on another line. Close every pair of markers on the same line they were opened.\n\
- Always close every pair of emphasis markers. Do not leave `**` or `*` unclosed.\n\
- Do not use bold or italic across line breaks. If you need to emphasize text on two lines, close the markers on the first line and reopen them on the second.\n"
, persona)
}

fn knowledge_priority_system_prompt(persona: &str) -> String {
    format!("\
You are an AI reading assistant embedded in a PDF reader. \
You help readers understand this document.\n\n\
Persona: {}\n\n\
You have access to:\n\
1. The text of the page the reader is currently viewing (CURRENT SECTION)\n\
2. Relevant excerpts retrieved from elsewhere in the document (RETRIEVED EXCERPTS)\n\
3. Your own general knowledge of the subject\n\n\
PRIORITY RULES — follow these strictly:\n\n\
- Answer primarily from your own general knowledge of the subject; be clear and correct.\n\n\
- The document excerpts are optional reference only — use a [n] citation only when a passage \
genuinely supports a specific point.\n\n\
- Do not fabricate; if unsure, say so. Keep the same Markdown/formatting rules.\n\n\
- If the document contradicts your general knowledge, prefer your general knowledge but note \
the contradiction if it is relevant.\n\n\
BEHAVIOR:\n\n\
- Keep responses concise and conversational. Aim for clarity over completeness.\n\n\
FORMATTING RULES — follow these strictly:\n\n\
- Use proper Markdown syntax for emphasis: write `**bold**` and `*italic*` with no spaces between the asterisks and the text.\n\
- Never start bold or italic on one line and close it on another line. Close every pair of markers on the same line they were opened.\n\
- Always close every pair of emphasis markers. Do not leave `**` or `*` unclosed.\n\
- Do not use bold or italic across line breaks. If you need to emphasize text on two lines, close the markers on the first line and reopen them on the second.\n"
, persona)
}

pub async fn build_grounding(
    db: &Db,
    file_hash: &str,
    question: &str,
    ctx: &ReaderContext,
    top_k: usize,
    embedder: &dyn EmbeddingProvider,
    persona: &str,
) -> Result<Grounding, LlmError> {
    let q_vectors = embedder.embed(&[question.to_string()], EmbedInput::Query).await?;
    let q_vector = q_vectors.first().ok_or_else(|| LlmError::Provider("Embedding provider returned no query vector".into()))?;

    let chunks = db
        .load_chunks(file_hash)
        .map_err(|e| LlmError::Provider(e.to_string()))?;

    let chunk_vectors: Vec<(crate::db::ChunkRow, Vec<f32>)> = chunks.into_iter().map(|row| {
        let mut vec = Vec::with_capacity(row.vector.len() / 4);
        for chunk in row.vector.chunks_exact(4) {
            vec.push(f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]));
        }
        (row, vec)
    }).collect();

    let mut scored: Vec<(f32, crate::db::ChunkRow)> = chunk_vectors
        .into_iter()
        .map(|(row, vec)| {
            let score = cosine_similarity(&vec, q_vector);
            (score, row)
        })
        .collect();

    scored.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());
    let max_score = scored.first().map(|(s, _)| *s).unwrap_or(0.0);

    let mut top_chunks: Vec<Chunk> = scored.iter().take(top_k).map(|(_, row)| {
        let quads: Vec<Quad> = serde_json::from_str(&row.quads_json).unwrap_or_default();
        Chunk {
            id: row.id.clone(),
            page: row.page,
            text: row.text.clone(),
            quads,
        }
    }).collect();

    // Force-inject current page chunks
    let current_page_chunks: Vec<Chunk> = scored
        .iter()
        .filter(|(_, row)| row.page == ctx.current_page)
        .map(|(_, row)| {
            let quads: Vec<Quad> = serde_json::from_str(&row.quads_json).unwrap_or_default();
            Chunk {
                id: row.id.clone(),
                page: row.page,
                text: row.text.clone(),
                quads,
            }
        })
        .collect();

    let mut seen = HashSet::new();
    for c in &top_chunks {
        seen.insert(c.id.clone());
    }
    for c in current_page_chunks {
        if seen.insert(c.id.clone()) {
            top_chunks.push(c);
        }
    }

    // Force-inject selection
    if let Some(sel_text) = &ctx.selection_text {
        if !sel_text.is_empty() {
            top_chunks.push(Chunk {
                id: format!("{}:selection", file_hash),
                page: ctx.current_page,
                text: sel_text.clone(),
                quads: ctx.selection_quads.clone(),
            });
        }
    }

    // Token budget
    let mut total_tokens = top_chunks.iter().map(|c| approximate_tokens(&c.text)).sum::<usize>();
    let budget = 4000;
    while total_tokens > budget && top_chunks.len() > 2 {
        let removable: Vec<usize> = top_chunks
            .iter()
            .enumerate()
            .filter(|(_, c)| c.page != ctx.current_page && !c.id.ends_with(":selection"))
            .map(|(i, _)| i)
            .collect();
        if removable.is_empty() {
            break;
        }
        let idx = *removable.last().unwrap();
        total_tokens -= approximate_tokens(&top_chunks[idx].text);
        top_chunks.remove(idx);
    }

    // Separate current-page / selection chunks from retrieved chunks.
    // Current-page chunks go first in both the sources list and the user prompt.
    let (current_chunks, rag_chunks): (Vec<Chunk>, Vec<Chunk>) = top_chunks
        .into_iter()
        .partition(|c| c.page == ctx.current_page || c.id.ends_with(":selection"));

    let mut ordered_chunks: Vec<Chunk> = Vec::new();
    ordered_chunks.extend(current_chunks.iter().cloned());
    ordered_chunks.extend(rag_chunks.iter().cloned());

    let sources: Vec<Source> = ordered_chunks
        .iter()
        .enumerate()
        .map(|(idx, chunk)| Source {
            idx: idx + 1,
            chunk_id: chunk.id.clone(),
            page: chunk.page,
            snippet: chunk.text.chars().take(300).collect(),
            quads: chunk.quads.clone(),
        })
        .collect();

    // --- System prompt: priority rules, no source list ---
    let system_prompt = rag_priority_system_prompt(persona);
    let knowledge_system_prompt = knowledge_priority_system_prompt(persona);

    // --- User prompt: current section first, RAG chunks next, question last ---
    let mut user_prompt = format!("The reader is currently on page {}.\n\n", ctx.current_page);

    user_prompt.push_str("---\n[CURRENT SECTION — highest priority. The reader is viewing this now.]\n\n");
    if current_chunks.is_empty() {
        user_prompt.push_str("(No text available for the current page.)\n");
    } else {
        for chunk in &current_chunks {
            let idx = ordered_chunks.iter().position(|c| c.id == chunk.id).unwrap_or(0) + 1;
            user_prompt.push_str(&format!("[{}] (p.{}) {}\n\n", idx, chunk.page, chunk.text));
        }
    }

    if !rag_chunks.is_empty() {
        user_prompt.push_str("---\n[RETRIEVED EXCERPTS — relevant passages from elsewhere in the document.]\n\n");
        for chunk in &rag_chunks {
            let idx = ordered_chunks.iter().position(|c| c.id == chunk.id).unwrap_or(0) + 1;
            user_prompt.push_str(&format!("[{}] (p.{}) {}\n\n", idx, chunk.page, chunk.text));
        }
    }

    user_prompt.push_str("---\n[READER'S QUESTION]\n");
    user_prompt.push_str(question);

    Ok(Grounding {
        system_prompt,
        knowledge_system_prompt,
        user_prompt,
        sources,
        max_score,
    })
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}
