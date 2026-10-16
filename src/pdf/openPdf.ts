import { readFile } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";
import { useReaderStore } from "../state/useReaderStore";
import { loadPdfFromArrayBuffer } from "./loader";

async function sha256(arrayBuffer: ArrayBuffer): Promise<string> {
  const hashBuffer = await crypto.subtle.digest("SHA-256", arrayBuffer);
  return Array.from(new Uint8Array(hashBuffer)).map((b) => b.toString(16).padStart(2, "0")).join("");
}

// Manual opens and OS requests share one serial queue. A failed file must not
// block later requests; this reader displays the last successfully opened PDF.
let pending: Promise<void> = Promise.resolve();
export function openPdfPath(path: string): Promise<void> {
  const job = pending.then(() => loadPdfPath(path));
  pending = job.catch(() => {});
  return job;
}

async function loadPdfPath(path: string): Promise<void> {
  const bytes = await readFile(path);
  const arrayBuffer = new Uint8Array(bytes).buffer;
  // PDF.js transfers this buffer to its worker; hash it before that transfer.
  const fileHash = await sha256(arrayBuffer);
  const doc = await loadPdfFromArrayBuffer(arrayBuffer);
  const previousDoc = useReaderStore.getState().pdfDoc;
  useReaderStore.getState().setPdfDoc(doc, path, fileHash);
  if (previousDoc) void previousDoc.destroy().catch(console.warn);

  const outline = await doc.getOutline().catch(() => []);
  if (useReaderStore.getState().pdfDoc !== doc) return;
  useReaderStore.getState().setOutline(outline ?? []);

  // Check if already indexed
  try {
    const config = useReaderStore.getState();
    const embedId = config.embedBackend === "remote"
      ? `remote:${config.embedBaseUrl.replace(/\/+$/, "")}/${config.embedModelRemote}`
      : `local:${config.embeddingModel}`;
    const indexed = (await invoke("is_indexed", { fileHash, embedId })) as boolean;
    if (useReaderStore.getState().pdfDoc !== doc) return;
    useReaderStore.getState().setIndexState(indexed ? "ready" : "idle");
  } catch {
    useReaderStore.getState().setIndexState("idle");
  }
}
