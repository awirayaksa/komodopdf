import { beforeEach, expect, it, vi } from "vitest";
import { readFile } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";
import { loadPdfFromArrayBuffer } from "../../src/pdf/loader";
import { openPdfPath } from "../../src/pdf/openPdf";
import { useReaderStore } from "../../src/state/useReaderStore";

vi.mock("@tauri-apps/plugin-fs", () => ({ readFile: vi.fn() }));
vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("../../src/pdf/loader", () => ({ loadPdfFromArrayBuffer: vi.fn() }));

beforeEach(() => {
  vi.resetAllMocks();
  useReaderStore.setState({ pdfDoc: null, totalPages: 0 });
  vi.mocked(readFile).mockResolvedValue(new Uint8Array([37, 80, 68, 70]));
  vi.mocked(invoke).mockResolvedValue(true);
});

function doc() {
  return { numPages: 2, getOutline: vi.fn(async () => []), destroy: vi.fn(async () => {}) };
}

it("reads native paths unchanged and hashes bytes before PDF.js transfers them", async () => {
  const pdf = doc();
  vi.mocked(loadPdfFromArrayBuffer).mockImplementation(async (buffer) => {
    structuredClone(buffer, { transfer: [buffer] });
    return pdf as never;
  });
  const path = "C:\\日本語 資料\\100%20 #1.pdf";
  await openPdfPath(path);
  expect(readFile).toHaveBeenCalledWith(path);
  expect(useReaderStore.getState().fileName).toBe(path);
  expect(useReaderStore.getState().fileHash).toMatch(/^[a-f0-9]{64}$/);
  expect(useReaderStore.getState().indexState).toBe("ready");
});

it("serializes manual/native opens and destroys the previous document; last file remains visible", async () => {
  let finish!: (pdf: never) => void;
  const first = doc(), second = doc();
  vi.mocked(loadPdfFromArrayBuffer)
    .mockImplementationOnce(() => new Promise((resolve) => { finish = resolve; }))
    .mockResolvedValueOnce(second as never);
  const a = openPdfPath("first.pdf"), b = openPdfPath("second.pdf");
  await vi.waitFor(() => expect(loadPdfFromArrayBuffer).toHaveBeenCalledOnce());
  expect(readFile).toHaveBeenCalledOnce();
  finish(first as never);
  await Promise.all([a, b]);
  expect(first.destroy).toHaveBeenCalledOnce();
  expect(useReaderStore.getState().pdfDoc).toBe(second);
  expect(useReaderStore.getState().fileName).toBe("second.pdf");
});

it("keeps the current document on read failure and recovers for the next PDF", async () => {
  const previous = doc(), next = doc();
  useReaderStore.setState({ pdfDoc: previous as never });
  vi.mocked(readFile).mockRejectedValueOnce(new Error("access denied"));
  await expect(openPdfPath("missing.pdf")).rejects.toThrow("access denied");
  expect(useReaderStore.getState().pdfDoc).toBe(previous);
  expect(previous.destroy).not.toHaveBeenCalled();
  vi.mocked(loadPdfFromArrayBuffer).mockResolvedValueOnce(next as never);
  await openPdfPath("valid.pdf");
  expect(useReaderStore.getState().pdfDoc).toBe(next);
});
