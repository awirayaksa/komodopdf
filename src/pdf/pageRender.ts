import type { PDFPageProxy } from "pdfjs-dist";

export async function renderPageToCanvas(page: PDFPageProxy, canvas: HTMLCanvasElement, scale: number, signal: AbortSignal) {
  signal.throwIfAborted();
  const viewport = page.getViewport({ scale });
  const dpr = window.devicePixelRatio || 1;
  // Private canvases prevent overlapping render tasks and blank frames during zoom.
  const buffer = document.createElement("canvas");
  buffer.width = Math.ceil(viewport.width * dpr);
  buffer.height = Math.ceil(viewport.height * dpr);
  const ctx = buffer.getContext("2d");
  if (!ctx) throw new Error("Could not get canvas context");
  const task = page.render({ canvasContext: ctx, viewport, transform: dpr === 1 ? undefined : [dpr, 0, 0, dpr, 0, 0] });
  const cancel = () => task.cancel();
  signal.addEventListener("abort", cancel, { once: true });
  try {
    await task.promise;
    signal.throwIfAborted();
    canvas.width = buffer.width;
    canvas.height = buffer.height;
    canvas.style.width = `${viewport.width}px`;
    canvas.style.height = `${viewport.height}px`;
    canvas.getContext("2d")?.drawImage(buffer, 0, 0);
  } finally {
    signal.removeEventListener("abort", cancel);
    buffer.width = buffer.height = 0;
  }
}
