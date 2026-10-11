<div align="center">

<img src="./komodopdf_logo.svg" alt="Komodo PDF" width="420" />

# Komodo PDF

**A local-first, private, open-source PDF reader with a grounded AI panel side-by-side.**

Ask questions about the document you're reading and get answers grounded in *its* text,
with page-cited sources you can click to jump to and highlight — never a black-box chatbot
guessing about a file it never read.

[![License: GPL v3](https://img.shields.io/badge/License-GPLv3-blue.svg)](./LICENSE)
[![Built with Tauri](https://img.shields.io/badge/built%20with-Tauri%202-24C8DB.svg)](https://tauri.app)
[![React](https://img.shields.io/badge/React-19-61DAFB.svg)](https://react.dev)
[![Rust](https://img.shields.io/badge/Rust-stable-CE412B.svg)](https://www.rust-lang.org)

</div>

---

## Why Komodo

Most "chat with your PDF" tools upload your document to someone else's server. Komodo doesn't —
**there is no Komodo server.** Your files stay on your machine. The AI panel talks only to the
provider *you* configure with *your own* API key, and if you pick local embeddings together with a
local chat endpoint, your document never leaves your laptop at all.

It's also a genuinely good reader first, AI assistant second: continuous scroll, zoom, outline
navigation, and text selection all work whether or not you ever turn the AI panel on.

## Features

### Reading
- 📖 Open any PDF with smooth continuous scroll
- 🔍 Zoom in/out, fit-width, fit-page
- 🗂️ Document outline / bookmarks navigation
- ✂️ Text selection with a real text layer (rendered via [pdf.js](https://mozilla.github.io/pdf.js/))
- 🏷️ Per-page "no text layer" badge so you know which pages are scanned images

### Grounded AI panel
- 💬 Ask questions about the open document and get **page-cited** answers
- 🎯 Click a `[n]` citation → scroll to the source passage and highlight it
- 🧠 **Retrieval-augmented** answers: Komodo indexes the document into a local vector store and
  retrieves the most relevant passages for each question
- 🧭 **Confidence routing**: when the document strongly answers a question, the assistant stays
  grounded in it; when it doesn't, it falls back to general knowledge — and tells you so
- 📍 Your **current page** and any **text selection** are always injected as high-priority context
- 🎭 Pick a **persona** ("Explain simply", etc.) to control the answer style

### Privacy & providers (bring your own key)
- 🔒 **No telemetry, no phone-home, no document upload to any Komodo server**
- 🖥️ **Local embeddings** via ONNX models downloaded on demand
  (`bge-small-en-v1.5` by default, `multilingual-e5-small` for CJK/multilingual)
- ☁️ Or **remote embeddings** from any OpenAI-compatible endpoint
- 🤖 **Chat** through any OpenAI-compatible provider (OpenAI, local servers, gateways…) using your key
- 🔑 API keys stored in the OS keyring, with an encrypted on-disk fallback so they survive restarts

## Tech stack

| Layer        | Tech                                                                 |
| ------------ | ------------------------------------------------------------------- |
| Shell        | [Tauri 2](https://tauri.app) (Rust)                                 |
| Frontend     | [React 19](https://react.dev) + TypeScript + [Vite](https://vite.dev) |
| State        | [Zustand](https://github.com/pmndrs/zustand)                        |
| PDF engine   | [pdf.js](https://mozilla.github.io/pdf.js/)                         |
| Markdown     | [react-markdown](https://github.com/remarkjs/react-markdown) + remark-gfm |
| Storage / RAG| SQLite vector store, cosine-similarity retrieval (Rust)            |
| Embeddings   | ONNX Runtime (local) and OpenAI-compatible HTTP (remote)           |

## Getting started

### Prerequisites
- [Node.js](https://nodejs.org) 18+
- [Rust](https://www.rust-lang.org/tools/install) (stable toolchain)
- Platform build dependencies required by Tauri — see the
  [Tauri prerequisites guide](https://tauri.app/start/prerequisites/)

### Run in development

```bash
npm install
npm run tauri dev
```

The first launch walks you through setup: choose a chat provider + model, pick local or remote
embeddings, and (for local embeddings) download the embedding model. Then open a PDF, let Komodo
index it, and start asking questions.

### Build a release

```bash
npm run tauri build
```

On Windows you can also use the bundled helper, which produces both `.msi` and `.exe` (NSIS)
installers:

```powershell
./scripts/build-release.ps1
```

## How it works

1. **Extract** — text and per-span coordinates are pulled from each PDF page in the frontend.
2. **Chunk** — pages are split into overlapping, sentence-aware chunks (~700 tokens) on the Rust side.
3. **Embed & store** — chunks are embedded (locally or remotely) and written to a local SQLite
   vector store, keyed by a hash of the file so re-opening is instant.
4. **Retrieve** — each question is embedded and scored against stored chunks by cosine similarity;
   the top matches, plus your current page and any selection, form the grounding context.
5. **Answer** — the context and question go to your configured chat model, which streams back a
   response with `[n]` citations that map to clickable, highlightable source passages.

## Privacy promise

Komodo's whole reason to exist is that **your documents don't go to a Komodo server — there is none.**
Network calls go *only* to the AI provider you explicitly configure. Contributions that add telemetry,
analytics, "phone-home," or any upload of document content will not be accepted. See
[CONTRIBUTING.md](./CONTRIBUTING.md#privacy-promise-dont-break-it).

## Regression tests

With Node.js 22.12 or newer, run `npm ci`, `npm test`, and `npm run build`.
Install the test browser with `npx playwright install chromium`, then run
`npm run test:e2e`. The browser tests use a synthetic PDF and mock Tauri IPC.
Rust regression tests run with `cargo test --manifest-path src-tauri/Cargo.toml --lib`.

## Contributing

Contributions are welcome! Please read [CONTRIBUTING.md](./CONTRIBUTING.md) first — Komodo uses the
[Developer Certificate of Origin](https://developercertificate.org) (sign off every commit with
`git commit -s`) and keeps a deliberately focused scope (it's a reader with a grounded AI panel, not
an Acrobat clone).

## License

Komodo PDF is licensed under the **GNU General Public License v3.0** — see [LICENSE](./LICENSE).
