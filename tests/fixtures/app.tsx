import React from "react";
import { createRoot } from "react-dom/client";
import { mockIPC } from "@tauri-apps/api/mocks";
import { emit } from "@tauri-apps/api/event";
import App from "../../src/App";
import { fixturePdf } from "./pdf";
import { useReaderStore } from "../../src/state/useReaderStore";
import { extractPageContent } from "../../src/pdf/extract";

const settings = { chatBaseUrl: "http://localhost:1234/v1", chatModel: "test",
  embedBackend: "local", embedModel: "bge-small-en-v1.5", topK: 6, persona: "Explain simply", configured: true, logLevel: "info" };
let asked = 0;
const nativePdfPaths: string[] = (window as any).__initialPdfPaths || [];
const readPaths: string[] = [];
(window as any).__readPdfPaths = readPaths;
(window as any).__readerStore = useReaderStore;
(window as any).__openNativePdfs = (paths: string[]) => {
  nativePdfPaths.push(...paths);
  return emit("pdf-open-requested");
};
mockIPC(async (cmd, args: any) => {
  switch (cmd) {
    case "take_pdf_open_requests": return nativePdfPaths.splice(0);
    case "get_settings": return settings;
    case "set_settings": Object.assign(settings, args.settings); return;
    case "has_api_key": return false;
    case "list_embedding_models": return [];
    case "test_connection": return { chatOk: true, embedOk: true };
    case "plugin:dialog|open": return "fixture.pdf";
    case "plugin:fs|read_file":
      readPaths.push(args.path);
      if (args.path.endsWith("missing.pdf")) throw new Error("File not found");
      return Array.from(fixturePdf());
    case "is_indexed": return true;
    case "ask": {
      asked++;
      const page = asked === 1 ? 2 : 8;
      const extraction = await extractPageContent(await useReaderStore.getState().pdfDoc!.getPage(page), page);
      const source = { idx: 1, chunk_id: `source-${page}`, page, snippet: extraction.spans[0].text, quads: extraction.spans[0].quads };
      args.channel.onmessage({ kind: "sources", data: [source] });
      args.channel.onmessage({ kind: "token", data: `Answer ${asked} [1].` });
      args.channel.onmessage({ kind: "done" });
      return;
    }
    default: throw new Error(`Unexpected IPC command: ${cmd}`);
  }
}, { shouldMockEvents: true });
createRoot(document.getElementById("root")!).render(<React.StrictMode><App /></React.StrictMode>);
