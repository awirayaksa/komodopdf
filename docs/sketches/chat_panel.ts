// DESIGN SKETCH — not wired up. Frontend (webview) side of the AI panel.
//
// Shows the two integration seams that matter most for v0.1:
//   1. Extracting text + on-page coordinates from PDF.js to feed the index.
//   2. The citation-click -> scroll-to + flash-highlight loop (SPEC §9.1, the magic).
//
// Backend calls go through Tauri's `invoke` / event channels so keys + network
// stay in Rust (SPEC §3).

import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";

// ---------- Shared shapes (mirror the Rust types) ----------

interface Quad { x0: number; y0: number; x1: number; y1: number; }
interface Citation { chunkId: string; page: number; snippet: string; quads: Quad[]; }
type ChatEvent =
  | { kind: "citations"; citations: Citation[] }
  | { kind: "token"; text: string }
  | { kind: "done" };

// ---------- 1. Extraction: PDF.js text items -> spans with quads ----------
// PDF.js gives each text item a transform matrix + width/height. Convert to a quad
// in PDF user space so the same coords work for retrieval citations AND highlights.

async function extractPage(pdfPage: any, pageNum: number) {
  const textContent = await pdfPage.getTextContent();
  const viewport = pdfPage.getViewport({ scale: 1 });
  const spans = textContent.items.map((item: any) => {
    const [a, b, , , e, f] = item.transform; // x-scale, skew, …, x, y
    const x0 = e;
    const y0 = f;
    const x1 = e + item.width;
    const y1 = f + (item.height ?? Math.hypot(a, b));
    return { text: item.str, quad: { x0, y0, x1, y1 } };
  });
  const hasText = spans.some((s: any) => s.text.trim().length > 0);
  return { page: pageNum, spans, hasText, viewport };
  // hasText === false  =>  show the per-page "no text layer / AI unavailable" badge.
}

// ---------- Asking a question ----------

async function ask(question: string, currentPage: number, selection?: { text: string; quads: Quad[] }) {
  // Stream events from the backend (grounding + provider call happen in Rust).
  const unlisten = await listen<ChatEvent>("komodo://chat", (e) => {
    const ev = e.payload;
    if (ev.kind === "citations") renderCitationChips(ev.citations);
    else if (ev.kind === "token") appendAssistantToken(ev.text);
    else if (ev.kind === "done") unlisten();
  });

  await invoke("ask", {
    question,
    ctx: {
      currentPage,
      selectionText: selection?.text ?? null,
      selectionQuads: selection?.quads ?? [],
    },
  });
}

// ---------- 2. THE MAGIC MOMENT: citation -> scroll + flash highlight ----------

function renderCitationChips(citations: Citation[]) {
  for (const c of citations) {
    const chip = document.createElement("button");
    chip.className = "citation-chip";
    chip.textContent = `p.${c.page}`;
    chip.title = c.snippet;
    chip.onclick = () => jumpToCitation(c); // <- the loop that sells the product
    citationsBar().appendChild(chip);
  }
}

async function jumpToCitation(c: Citation) {
  await scrollToPage(c.page);            // bring the page into view
  const overlay = highlightLayerFor(c.page);
  for (const q of c.quads) {
    const el = quadToScreenRect(q, c.page); // apply current zoom/viewport transform
    el.classList.add("flash-highlight");    // CSS: brief pulse, then fade to persistent tint
    overlay.appendChild(el);
  }
}

// ---------- stubs to implement against your renderer ----------
function appendAssistantToken(_t: string) {}
function citationsBar(): HTMLElement { return document.body; }
function scrollToPage(_p: number): Promise<void> { return Promise.resolve(); }
function highlightLayerFor(_p: number): HTMLElement { return document.body; }
function quadToScreenRect(_q: Quad, _page: number): HTMLElement { return document.createElement("div"); }
