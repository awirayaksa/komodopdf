import { open } from "@tauri-apps/plugin-dialog";
import { readFile } from "@tauri-apps/plugin-fs";
import { invoke } from "@tauri-apps/api/core";
import { useState } from "react";
import { useReaderStore } from "../state/useReaderStore";
import { loadPdfFromArrayBuffer } from "../pdf/loader";

export default function Toolbar() {
  const {
    currentPage,
    totalPages,
    displayScale,
    fitMode,
    setScale,
    setFitMode,
    toggleOutline,
  } = useReaderStore();

  const hasDoc = totalPages > 0;
  const [opening, setOpening] = useState(false);
  const [openError, setOpenError] = useState<string | null>(null);

  async function sha256(arrayBuffer: ArrayBuffer): Promise<string> {
    const hashBuffer = await crypto.subtle.digest("SHA-256", arrayBuffer);
    const hashArray = Array.from(new Uint8Array(hashBuffer));
    return hashArray.map((b) => b.toString(16).padStart(2, "0")).join("");
  }

  async function handleOpen() {
    setOpenError(null);
    setOpening(true);
    try {
    const selected = await open({
      multiple: false,
      directory: false,
      filters: [{ name: "PDF", extensions: ["pdf"] }],
    });
    if (!selected || Array.isArray(selected)) return;

    const bytes = await readFile(selected);
    const arrayBuffer = new Uint8Array(bytes).buffer;
    // PDF.js transfers this buffer to its worker; hash it before that transfer.
    const fileHash = await sha256(arrayBuffer);
    const doc = await loadPdfFromArrayBuffer(arrayBuffer);
    const previousDoc = useReaderStore.getState().pdfDoc;
    useReaderStore.getState().setPdfDoc(doc, selected, fileHash);
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
    } catch (e) {
      setOpenError(String(e));
    } finally {
      setOpening(false);
    }
  }

  return (
    <header
      style={{
        display: "flex",
        alignItems: "center",
        gap: 8,
        padding: "8px 12px",
        backgroundColor: "var(--surface-raised)",
        borderBottom: "1px solid var(--border)",
        height: 48,
        flexShrink: 0,
      }}
    >
      <div style={{ display: "flex", alignItems: "center", gap: 6 }}>
        <svg width="24" height="24" viewBox="0 0 400 400" xmlns="http://www.w3.org/2000/svg" aria-hidden="true">
          <rect width="400" height="400" fill="var(--surface)" rx="48"/>
          <ellipse cx="160" cy="210" rx="72" ry="38" fill="var(--accent-hover)"/>
          <ellipse cx="232" cy="198" rx="38" ry="28" fill="var(--accent-hover)"/>
          <polygon points="265,193 295,198 265,205" fill="var(--accent-press)"/>
          <circle cx="240" cy="191" r="6" fill="var(--surface)"/>
          <circle cx="242" cy="190" r="2" fill="var(--ai-accent)"/>
          <rect x="138" y="163" width="46" height="54" rx="5" fill="var(--accent-tint)" opacity="0.95"/>
          <rect x="138" y="163" width="46" height="54" rx="5" fill="none" stroke="var(--accent)" strokeWidth="1.5"/>
        </svg>
        <span style={{ fontWeight: 600, fontSize: 15, color: "var(--text)", letterSpacing: -0.3 }}>
          Komodo PDF
        </span>
      </div>

      <button
        onClick={toggleOutline}
        title="Toggle outline"
        style={{
          padding: "6px 10px",
          borderRadius: 6,
          backgroundColor: "var(--surface)",
          border: "1px solid var(--border)",
          marginLeft: 8,
        }}
      >
        ☰
      </button>

      <button
        onClick={handleOpen}
        disabled={opening}
        className="btn-accent"
        style={{
          padding: "6px 14px",
          borderRadius: 6,
          fontWeight: 500,
        }}
      >
        {opening ? "Opening…" : "Open PDF"}
      </button>
      {openError && <span role="alert" title={openError} style={{ color: "#f59e0b", fontSize: 12 }}>Could not open PDF: {openError}</span>}


      {hasDoc && (
        <>
          <div style={{ display: "flex", alignItems: "center", gap: 4, marginLeft: 8 }}>
            <button
              onClick={() => useReaderStore.getState().prevPage()}
              disabled={currentPage <= 1}
              style={{
                padding: "4px 8px",
                borderRadius: 4,
                backgroundColor: "var(--surface)",
                border: "1px solid var(--border)",
                opacity: currentPage <= 1 ? 0.4 : 1,
              }}
            >
              ‹
            </button>
            <span style={{ fontVariantNumeric: "tabular-nums", minWidth: 60, textAlign: "center" }}>
              {currentPage} / {totalPages}
            </span>
            <button
              onClick={() => useReaderStore.getState().nextPage()}
              disabled={currentPage >= totalPages}
              style={{
                padding: "4px 8px",
                borderRadius: 4,
                backgroundColor: "var(--surface)",
                border: "1px solid var(--border)",
                opacity: currentPage >= totalPages ? 0.4 : 1,
              }}
            >
              ›
            </button>
          </div>

          <div style={{ display: "flex", alignItems: "center", gap: 4, marginLeft: "auto" }}>
            <button
              onClick={() => setScale(displayScale * 1.2)}
              title="Zoom in"
              style={{
                padding: "4px 8px",
                borderRadius: 4,
                backgroundColor: "var(--surface)",
                border: "1px solid var(--border)",
              }}
            >
              +
            </button>
            <span style={{ fontVariantNumeric: "tabular-nums", minWidth: 48, textAlign: "center" }}>
              {fitMode ? "Fit" : `${Math.round(displayScale * 100)}%`}
            </span>
            <button
              onClick={() => setScale(Math.max(0.25, displayScale / 1.2))}
              title="Zoom out"
              style={{
                padding: "4px 8px",
                borderRadius: 4,
                backgroundColor: "var(--surface)",
                border: "1px solid var(--border)",
              }}
            >
              −
            </button>
            <button
              onClick={() => setFitMode(fitMode === "width" ? null : "width")}
              title="Fit width"
              aria-pressed={fitMode === "width"}
              className={fitMode === "width" ? "btn-accent-active" : ""}
              style={{
                padding: "4px 10px",
                borderRadius: 4,
                backgroundColor: fitMode === "width" ? undefined : "var(--surface)",
                border: "1px solid var(--border)",
                marginLeft: 4,
              }}
            >
              ⇤
            </button>
            <button
              onClick={() => setFitMode(fitMode === "page" ? null : "page")}
              title="Fit page"
              aria-pressed={fitMode === "page"}
              className={fitMode === "page" ? "btn-accent-active" : ""}
              style={{
                padding: "4px 10px",
                borderRadius: 4,
                backgroundColor: fitMode === "page" ? undefined : "var(--surface)",
                border: "1px solid var(--border)",
              }}
            >
              ☐
            </button>
          </div>
        </>
      )}
    </header>
  );
}
