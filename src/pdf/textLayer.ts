import { TextLayer } from "pdfjs-dist";
import type { PDFPageProxy } from "pdfjs-dist";
import "pdfjs-dist/web/pdf_viewer.css";

export async function buildTextLayer(page: PDFPageProxy, container: HTMLDivElement, scale: number, signal: AbortSignal): Promise<boolean> {
  const textContent = await page.getTextContent();
  signal.throwIfAborted();
  const buffer = document.createElement("div");
  buffer.className = "textLayer pdf-text-layer";
  buffer.style.setProperty("--scale-factor", String(scale));
  const layer = new TextLayer({ textContentSource: textContent, container: buffer, viewport: page.getViewport({ scale }) });
  const cancel = () => layer.cancel();
  signal.addEventListener("abort", cancel, { once: true });
  try {
    await layer.render();
    signal.throwIfAborted();
    container.style.setProperty("--scale-factor", String(scale));
    container.replaceChildren(...Array.from(buffer.childNodes));
    return textContent.items.some((item) => "str" in item && item.str.trim().length > 0);
  } finally {
    signal.removeEventListener("abort", cancel);
  }
}
