import { expect, it } from "vitest";
import type { PDFPageProxy } from "pdfjs-dist";
import { extractPageContent } from "../../src/pdf/extract";

it("returns highlight boxes in canvas coordinates rather than PDF bottom-left coordinates", async () => {
  const page = {
    getTextContent: async () => ({ items: [{ str: "Top of page", transform: [24, 0, 0, 24, 50, 720], width: 120, height: 24 }] }),
    getViewport: () => ({ convertToViewportRectangle: ([x0, y0, x1, y1]: number[]) => [x0, 800 - y0, x1, 800 - y1] }),
  } as unknown as PDFPageProxy;
  const extraction = await extractPageContent(page, 1);
  expect(extraction.spans[0].quads).toEqual([{ x0: 50, y0: 56, x1: 170, y1: 80 }]);
});
