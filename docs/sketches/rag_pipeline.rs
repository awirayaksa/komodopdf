//! DESIGN SKETCH — not wired up. The RAG pipeline that grounds answers in the book.
//!
//! Two phases:
//!   A. INDEX (once per document, on open / in background): extract text + coords,
//!      chunk, embed, store in a local vector table.
//!   B. QUERY (per question): retrieve top-k + FORCE-INJECT current page/selection,
//!      assemble the grounding prompt, hand to the provider, return citations.
//!
//! Key principle (SPEC §3): the user's current page and any selection are ALWAYS in
//! context, regardless of what similarity search returns. Pure top-k frequently misses
//! the page the user is literally looking at — and "explain THIS" is the #1 use case.

use crate::llm::{Citation, EmbeddingProvider, Quad};

// ---------- Index-side types ----------

/// One extracted unit of the PDF, with the coordinates needed to highlight it later.
/// PDF.js (frontend) emits text items with transforms; we carry the resulting quads
/// down so a citation can scroll-to + flash the exact span (SPEC §9.1).
pub struct TextSpan {
    pub page: u32,        // 1-based
    pub text: String,
    pub quads: Vec<Quad>, // PDF user-space rectangles for this span
}

pub struct Chunk {
    pub id: String,        // stable: e.g. "{file_hash}:{page}:{ordinal}"
    pub page: u32,
    pub text: String,
    pub quads: Vec<Quad>,  // union of the spans that formed this chunk
}

/// Per-page extraction result. Drives the "no text layer" banner (SPEC §4):
/// detect emptiness PER PAGE, not per document (mixed scans are common).
pub struct PageExtraction {
    pub page: u32,
    pub spans: Vec<TextSpan>,
    pub has_text: bool,    // false => scanned/image page; mark "AI unavailable here"
}

// ---------- INDEX ----------

/// Chunking: ~500–800 tokens with small overlap is a fine v0.1 default. Prefer
/// breaking on paragraph/sentence boundaries within a page; do NOT span pages in a
/// single chunk (keeps citations to one page and highlighting simple).
pub fn chunk_pages(pages: &[PageExtraction], _target_tokens: usize) -> Vec<Chunk> {
    // 1. Concatenate spans per page in reading order.
    // 2. Split into ~target_tokens windows on sentence boundaries.
    // 3. Carry the union of contributing span quads onto each chunk.
    todo!("group spans -> windows; keep page + quads")
}

pub struct IndexedDoc {
    pub file_hash: String, // keys the vector table AND the annotation store (SPEC §3)
    pub embed_dims: usize,
}

/// INDEX phase. Idempotent: if file_hash already indexed with same embedding model,
/// skip. Run off the UI thread; show progress for big books.
pub async fn index_document(
    file_hash: &str,
    pages: &[PageExtraction],
    embedder: &dyn EmbeddingProvider,
    // store: &VectorStore,   // SQLite + sqlite-vec / LanceDB
) -> anyhow::Result<IndexedDoc> {
    let chunks = chunk_pages(pages, 700);
    let texts: Vec<String> = chunks.iter().map(|c| c.text.clone()).collect();
    let _vectors = embedder.embed(&texts).await?;
    // store.upsert(file_hash, &chunks, &_vectors, embedder.id(), embedder.dimensions())?;
    Ok(IndexedDoc { file_hash: file_hash.to_string(), embed_dims: embedder.dimensions() })
}

// ---------- QUERY ----------

/// What the UI sends with each question: where the reader currently is, and what
/// (if anything) is selected. This is the non-negotiable grounding floor.
pub struct ReaderContext {
    pub current_page: u32,
    pub selection_text: Option<String>,
    pub selection_quads: Vec<Quad>,
}

pub struct Grounding {
    pub system_prompt: String,   // assembled context + citing instructions
    pub citations: Vec<Citation>, // everything the answer may reference
}

/// QUERY phase. Returns the grounding to feed into LlmProvider::chat_stream.
pub async fn build_grounding(
    file_hash: &str,
    question: &str,
    ctx: &ReaderContext,
    top_k: usize,                 // SPEC §6 — user-tunable retrieval depth
    embedder: &dyn EmbeddingProvider,
    // store: &VectorStore,
) -> anyhow::Result<Grounding> {
    // 1. Retrieve by similarity.
    let _q = embedder.embed(&[question.to_string()]).await?;
    // let mut hits = store.search(file_hash, &_q[0], top_k)?;
    let mut hits: Vec<Chunk> = Vec::new(); // placeholder

    // 2. FORCE-INJECT the current page + selection so "explain this" always works,
    //    even if similarity search didn't surface them. Dedupe against `hits`.
    //    - always include the current page's chunk(s)
    //    - if there is a selection, include it as a high-priority span
    let _ = (&mut hits, ctx);

    // 3. Assemble: numbered sources [1],[2]… with page numbers; instruct the model
    //    to cite ONLY these sources by number and to say "not found in this document"
    //    rather than invent. (Hallucination guard — SPEC thesis.)
    let system_prompt = assemble_prompt(&hits, ctx, question);

    // 4. Map chunks -> Citations so the UI can render clickable, highlightable chips.
    let citations = hits
        .iter()
        .map(|c| Citation {
            chunk_id: c.id.clone(),
            page: c.page,
            snippet: c.text.chars().take(160).collect(),
            quads: c.quads.clone(),
        })
        .collect();

    Ok(Grounding { system_prompt, citations })
}

fn assemble_prompt(_hits: &[Chunk], _ctx: &ReaderContext, _question: &str) -> String {
    // System instructions + "SOURCES:\n[1] (p.X) …" + the current selection block.
    // Keep citing rules strict; this is what makes answers trustworthy.
    todo!("format grounded system prompt with numbered, page-tagged sources")
}

// ---------- Flow summary ----------
// open PDF
//   -> frontend (PDF.js) extracts PageExtraction[] (text + quads)
//   -> index_document() embeds + stores  [background, per file_hash]
// ask question
//   -> build_grounding() retrieves top_k + forces current page/selection
//   -> LlmProvider::chat_stream(req, citations) streams the answer
//   -> UI renders tokens + citation chips
//   -> click chip -> scroll to page + flash quads   ← the magic moment (SPEC §9.1)
