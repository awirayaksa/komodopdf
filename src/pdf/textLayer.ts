import { TextLayer } from "pdfjs-dist";
import type { PDFPageProxy } from "pdfjs-dist";

export async function buildTextLayer(
  page: PDFPageProxy,
  container: HTMLDivElement,
  scale: number
): Promise<boolean> {
  const textContent = await page.getTextContent();
  container.innerHTML = "";

  const viewport = page.getViewport({ scale });

  const textLayer = new TextLayer({
    textContentSource: textContent,
    container,
    viewport,
  });

  await textLayer.render();

  const spans = container.querySelectorAll("span");
  for (const span of spans) {
    (span as HTMLElement).style.userSelect = "text";
    (span as HTMLElement).style.color = "transparent";
    (span as HTMLElement).style.cursor = "text";
  }

  let hasAnyText = false;
  for (const item of textContent.items) {
    if ("str" in item && item.str && item.str.trim().length > 0) {
      hasAnyText = true;
      break;
    }
  }

  return hasAnyText;
}
