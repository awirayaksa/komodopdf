import type { PDFDocumentProxy, PDFPageProxy } from "pdfjs-dist";

export interface Quad {
  x0: number;
  y0: number;
  x1: number;
  y1: number;
}

export interface TextSpan {
  text: string;
  quads: Quad[];
}

export interface PageContent {
  page: number;
  spans: TextSpan[];
  hasText: boolean;
}

interface GlyphRun {
  str: string;
  x0: number;
  y0: number;
  x1: number;
  y1: number;
  fontSize: number;
}

export async function extractPageContent(
  page: PDFPageProxy,
  pageNum: number
): Promise<PageContent> {
  const textContent = await page.getTextContent();

  const runs: GlyphRun[] = [];
  let hasText = false;

  for (const item of textContent.items) {
    if (!("str" in item)) continue;
    const str = (item as any).str as string;
    if (!str || str.trim().length === 0) continue;

    hasText = true;

    const transform = (item as any).transform as number[];
    const width = (item as any).width as number;
    const height = (item as any).height as number;

    const a = transform[0];
    const b = (transform[1] ?? 0) as number;
    const e = transform[4];
    const f = transform[5];

    const fontSize = Math.hypot(a, b);
    const x0 = e;
    const y0 = f;
    const x1 = e + width;
    const y1 = f + (height || fontSize);

    runs.push({ str, x0, y0, x1, y1, fontSize });
  }

  if (!hasText || runs.length === 0) {
    return { page: pageNum, spans: [], hasText };
  }

  // Determine a vertical tolerance for grouping runs into lines.
  const sortedSizes = runs.map((r) => r.fontSize).sort((a, b) => a - b);
  const medianFontSize = sortedSizes[Math.floor(sortedSizes.length / 2)] || 12;
  const yTolerance = medianFontSize * 0.35;

  // PDF coordinates: y increases upward, so read top-to-bottom by sorting descending y.
  runs.sort((a, b) => b.y0 - a.y0);

  const lines: GlyphRun[][] = [];
  let currentLine: GlyphRun[] = [runs[0]];

  for (let i = 1; i < runs.length; i++) {
    const last = currentLine[currentLine.length - 1];
    const curr = runs[i];

    if (Math.abs(curr.y0 - last.y0) <= yTolerance) {
      currentLine.push(curr);
    } else {
      lines.push(currentLine);
      currentLine = [curr];
    }
  }
  lines.push(currentLine);

  const spans: TextSpan[] = [];

  for (const line of lines) {
    // Read each line left-to-right.
    line.sort((a, b) => a.x0 - b.x0);

    let lineX0 = line[0].x0;
    let lineY0 = line[0].y0;
    let lineX1 = line[0].x1;
    let lineY1 = line[0].y1;

    let text = line[0].str;

    for (let i = 1; i < line.length; i++) {
      const prev = line[i - 1];
      const curr = line[i];

      const gap = curr.x0 - prev.x1;
      const threshold = prev.fontSize * 0.18;

      // Insert a space only when the physical gap between runs is large enough
      // to be a word break, but not so small that it is just kerning/tracking.
      if (gap > threshold) {
        text += " ";
      }

      text += curr.str;

      lineX0 = Math.min(lineX0, curr.x0);
      lineY0 = Math.min(lineY0, curr.y0);
      lineX1 = Math.max(lineX1, curr.x1);
      lineY1 = Math.max(lineY1, curr.y1);
    }

    // Append a newline so concatenated page text preserves line structure.
    spans.push({
      text: text + "\n",
      quads: [{ x0: lineX0, y0: lineY0, x1: lineX1, y1: lineY1 }],
    });
  }

  return { page: pageNum, spans, hasText };
}

export async function extractAllPages(
  pdfDoc: PDFDocumentProxy
): Promise<PageContent[]> {
  const results: PageContent[] = [];
  for (let i = 1; i <= pdfDoc.numPages; i++) {
    const page = await pdfDoc.getPage(i);
    const content = await extractPageContent(page, i);
    results.push(content);
  }
  return results;
}
