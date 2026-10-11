import { create } from "zustand";
import type { PDFDocumentProxy } from "pdfjs-dist";

export interface Quad {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

interface OutlineItem {
  title: string;
  dest: string | any[] | null;
  items: OutlineItem[];
}

export interface Source {
  idx: number;
  chunkId: string;
  page: number;
  snippet: string;
  quads: Quad[];
}

export interface Message {
  role: "user" | "assistant";
  content: string;
  sources?: Source[];
  citations?: number[];
}

interface ReaderState {
  pdfDoc: PDFDocumentProxy | null;
  fileName: string;
  totalPages: number;
  currentPage: number;
  navigationRequest: { page: number; id: number };
  scale: number;
  displayScale: number;
  fitMode: "width" | "page" | null;
  outline: OutlineItem[];
  hasTextMap: Map<number, boolean>;
  selection: string;
  selectionQuads: Quad[];
  outlineOpen: boolean;

  // AI
  fileHash: string | null;
  indexState: "idle" | "extracting" | "indexing" | "ready";
  indexProgress: { done: number; total: number } | null;
  messages: Message[];
  streaming: boolean;
  activeHighlight: { page: number; quads: Quad[] } | null;
  pendingQuestion: string | null;
  activeModal: "none" | "setup" | "indexConfirm" | "indexing";

  // Config mirror
  configured: boolean;
  apiKeyPresent: boolean;
  baseUrl: string;
  embedBackend: "local" | "remote";
  embedBaseUrl: string;
  embedModelRemote: string;
  chatModel: string;
  embeddingModel: string;
  topK: number;
  persona: string;

  setPdfDoc: (doc: PDFDocumentProxy | null, fileName?: string, fileHash?: string | null) => void;
  setCurrentPage: (page: number) => void;
  updateCurrentPage: (page: number) => void;
  setScale: (scale: number) => void;
  setDisplayScale: (scale: number) => void;
  setFitMode: (mode: "width" | "page" | null) => void;
  setOutline: (outline: OutlineItem[]) => void;
  setHasText: (pageNum: number, hasText: boolean) => void;
  setSelection: (text: string, quads?: Quad[]) => void;
  toggleOutline: () => void;
  nextPage: () => void;
  prevPage: () => void;

  setFileHash: (hash: string | null) => void;
  setIndexState: (state: ReaderState["indexState"]) => void;
  setIndexProgress: (progress: ReaderState["indexProgress"]) => void;
  addMessage: (message: Message) => void;
  appendToken: (token: string) => void;
  setStreaming: (streaming: boolean) => void;
  setActiveHighlight: (highlight: ReaderState["activeHighlight"]) => void;
  clearHighlight: () => void;
  setPendingQuestion: (q: string | null) => void;
  setActiveModal: (modal: ReaderState["activeModal"]) => void;
  setConfigured: (v: boolean) => void;
  setApiKeyPresent: (v: boolean) => void;
  setConfig: (cfg: Partial<Pick<ReaderState, "baseUrl" | "chatModel" | "embedBackend" | "embedBaseUrl" | "embedModelRemote" | "embeddingModel" | "topK" | "persona">>) => void;
  clearChat: () => void;
}

export const useReaderStore = create<ReaderState>((set, get) => ({
  pdfDoc: null,
  fileName: "",
  totalPages: 0,
  currentPage: 1,
  navigationRequest: { page: 1, id: 0 },
  scale: 1.5,
  displayScale: 1.5,
  fitMode: null,
  outline: [],
  hasTextMap: new Map(),
  selection: "",
  selectionQuads: [],
  outlineOpen: true,

  fileHash: null,
  indexState: "idle",
  indexProgress: null,
  messages: [],
  streaming: false,
  activeHighlight: null,
  pendingQuestion: null,
  activeModal: "none",

  configured: false,
  apiKeyPresent: false,
  baseUrl: "https://api.openai.com/v1",
  embedBackend: "local",
  embedBaseUrl: "",
  embedModelRemote: "",
  chatModel: "gpt-4o-mini",
  embeddingModel: "bge-small-en-v1.5",
  topK: 6,
  persona: "Explain simply",

  setPdfDoc: (doc, fileName = "", fileHash = null) => {
    if (doc) {
      set({
        pdfDoc: doc,
        fileName,
        fileHash,
        totalPages: doc.numPages,
        currentPage: 1,
        navigationRequest: { page: 1, id: get().navigationRequest.id + 1 },
        outline: [],
        hasTextMap: new Map(),
        selection: "",
        selectionQuads: [],
        indexState: "idle",
        indexProgress: null,
        messages: [],
        streaming: false,
        activeHighlight: null,
        pendingQuestion: null,
      });
    } else {
      set({
        pdfDoc: null,
        fileName: "",
        fileHash: null,
        totalPages: 0,
        currentPage: 1,
        outline: [],
        hasTextMap: new Map(),
        selection: "",
        selectionQuads: [],
        indexState: "idle",
        indexProgress: null,
        messages: [],
        streaming: false,
        activeHighlight: null,
        pendingQuestion: null,
      });
    }
  },

  setCurrentPage: (page) => {
    if (!Number.isFinite(page)) return;
    const target = Math.max(1, Math.min(Math.trunc(page), get().totalPages || 1));
    set({ currentPage: target, navigationRequest: { page: target, id: get().navigationRequest.id + 1 } });
  },
  updateCurrentPage: (page) => set({ currentPage: page }),
  setScale: (scale) => set({ scale: Math.max(0.25, Math.min(5, scale)), fitMode: null }),
  setDisplayScale: (displayScale) => set({ displayScale }),
  setFitMode: (mode) => set({ fitMode: mode }),
  setOutline: (outline) => set({ outline }),
  setHasText: (pageNum, hasText) => {
    const map = new Map(get().hasTextMap);
    map.set(pageNum, hasText);
    set({ hasTextMap: map });
  },
  setSelection: (text, quads = []) => set({ selection: text, selectionQuads: quads }),
  toggleOutline: () => set((s) => ({ outlineOpen: !s.outlineOpen })),
  nextPage: () => {
    const s = get();
    if (s.currentPage < s.totalPages) s.setCurrentPage(s.currentPage + 1);
  },
  prevPage: () => {
    const s = get();
    if (s.currentPage > 1) s.setCurrentPage(s.currentPage - 1);
  },

  setFileHash: (hash) => set({ fileHash: hash }),
  setIndexState: (state) => set({ indexState: state }),
  setIndexProgress: (progress) => set({ indexProgress: progress }),
  addMessage: (message) => set((s) => ({ messages: [...s.messages, message] })),
  appendToken: (token) =>
    set((s) => {
      const msgs = [...s.messages];
      const last = msgs[msgs.length - 1];
      if (last && last.role === "assistant") {
        last.content += token;
      } else {
        msgs.push({ role: "assistant", content: token });
      }
      return { messages: msgs };
    }),
  setStreaming: (streaming) => set({ streaming }),
  setActiveHighlight: (highlight) => set({ activeHighlight: highlight }),
  clearHighlight: () => set({ activeHighlight: null }),
  setPendingQuestion: (q) => set({ pendingQuestion: q }),
  setActiveModal: (modal) => set({ activeModal: modal }),
  setConfigured: (v) => set({ configured: v }),
  setApiKeyPresent: (v) => set({ apiKeyPresent: v }),
  setConfig: (cfg) => {
    const previous = get();
    const backend = cfg.embedBackend ?? previous.embedBackend;
    const changed = backend !== previous.embedBackend || (backend === "local"
      ? cfg.embeddingModel !== undefined && cfg.embeddingModel !== previous.embeddingModel
      : (cfg.embedBaseUrl !== undefined && cfg.embedBaseUrl !== previous.embedBaseUrl)
        || (cfg.embedModelRemote !== undefined && cfg.embedModelRemote !== previous.embedModelRemote));
    set({ ...cfg, ...(changed ? { indexState: "idle" as const } : {}) });
  },
  clearChat: () => set({ messages: [] }),
}));
