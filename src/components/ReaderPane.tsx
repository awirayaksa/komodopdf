import { memo, useEffect, useLayoutEffect, useRef, useCallback, useState } from "react";
import type { PDFDocumentProxy } from "pdfjs-dist";
import { useReaderStore } from "../state/useReaderStore";
import { renderPageToCanvas } from "../pdf/pageRender";
import { buildTextLayer } from "../pdf/textLayer";

interface PageSize { width: number; height: number }

function fittedScale(size: PageSize, bounds: PageSize, fitMode: "width" | "page" | null, scale: number) {
  const widthScale = Math.max(1, bounds.width - 32) / size.width;
  return fitMode === "width" ? widthScale : fitMode === "page"
    ? Math.min(widthScale, Math.max(1, bounds.height - 32) / size.height) : scale;
}

const PdfPageView = memo(function PdfPageView({ pdfDoc, pageNum, scale, size, root }: {
  pdfDoc: PDFDocumentProxy; pageNum: number; scale: number; size: PageSize; root: HTMLDivElement;
}) {
  const highlight = useReaderStore((s) => s.activeHighlight?.page === pageNum ? s.activeHighlight : null);
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const textRef = useRef<HTMLDivElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const [visible, setVisible] = useState(false);
  const [hasText, setHasText] = useState<boolean | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    const observer = new IntersectionObserver(([entry]) => setVisible(entry.isIntersecting), { root, rootMargin: "800px 0px", threshold: 0 });
    observer.observe(wrapperRef.current!);
    return () => observer.disconnect();
  }, [root]);

  useEffect(() => {
    const canvas = canvasRef.current!;
    const text = textRef.current!;
    if (!visible) {
      canvas.width = canvas.height = 0;
      text.replaceChildren();
      return;
    }
    const controller = new AbortController();
    setError(null);
    (async () => {
      const page = await pdfDoc.getPage(pageNum);
      controller.signal.throwIfAborted();
      await renderPageToCanvas(page, canvas, scale, controller.signal);
      const result = await buildTextLayer(page, text, scale, controller.signal);
      controller.signal.throwIfAborted();
      setHasText(result);
      useReaderStore.getState().setHasText(pageNum, result);
    })().catch((e: unknown) => { if (!controller.signal.aborted) setError(String(e)); });
    return () => controller.abort();
  }, [pdfDoc, pageNum, scale, visible]);

  useEffect(() => {
    if (!highlight) return;
    const timer = setTimeout(() => {
      if (useReaderStore.getState().activeHighlight === highlight) useReaderStore.getState().clearHighlight();
    }, 3000);
    return () => clearTimeout(timer);
  }, [highlight]);

  return (
    <div ref={wrapperRef} style={{ position: "relative", width: size.width * scale, height: size.height * scale,
      margin: "0 auto", boxShadow: "0 2px 8px rgba(0,0,0,0.2)", background: "var(--page-bg)" }}>
      <canvas ref={canvasRef} style={{ display: "block", width: "100%", height: "100%" }} />
      <div ref={textRef} className="textLayer pdf-text-layer" onMouseUp={() => {
        useReaderStore.getState().setSelection(window.getSelection()?.toString() || "");
      }} />
      <div style={{ position: "absolute", top: 4, right: 8, fontSize: 11,
        color: "var(--text-muted)", pointerEvents: "none" }}>{pageNum}</div>
      {highlight?.quads.map((q, i) => (
        <div key={i} style={{ position: "absolute", left: Math.min(q.x0, q.x1) * scale,
          top: Math.min(q.y0, q.y1) * scale, width: Math.abs(q.x1 - q.x0) * scale,
          height: Math.abs(q.y1 - q.y0) * scale, background: "rgba(245,158,11,0.35)",
          border: "1px solid rgba(245,158,11,0.8)", pointerEvents: "none", animation: "flash-amber 3s ease-out forwards" }} />
      ))}
      {(error || hasText === false) && <div role={error ? "alert" : undefined} style={{ position: "absolute", bottom: 8,
        left: 8, right: 8, padding: 4, fontSize: 12, background: "var(--badge-bg)", color: "var(--text)" }}>
        {error ? `Page could not be rendered: ${error}` : "No text layer — AI unavailable here"}
      </div>}
    </div>
  );
});

export default function ReaderPane() {
  const pdfDoc = useReaderStore((s) => s.pdfDoc);
  const scale = useReaderStore((s) => s.scale);
  const currentPage = useReaderStore((s) => s.currentPage);
  const fitMode = useReaderStore((s) => s.fitMode);
  const navigation = useReaderStore((s) => s.navigationRequest);
  const scrollRef = useRef<HTMLDivElement>(null);
  const [layout, setLayout] = useState<{ doc: PDFDocumentProxy; pages: PageSize[] } | null>(null);
  const [bounds, setBounds] = useState({ width: 0, height: 0 });
  const [error, setError] = useState<string | null>(null);
  const frameRef = useRef(0);
  const navigatingRef = useRef(false);
  const positionRef = useRef({ top: 0, left: 0 });
  const geometryRef = useRef<{ doc: PDFDocumentProxy; navigation: typeof navigation;
    pages: { top: number; height: number }[] } | null>(null);

  useEffect(() => {
    if (!pdfDoc) return;
    let cancelled = false;
    setError(null);
    // Reserve actual page sizes first so lazy rendering never changes scroll height.
    (async () => {
      const pages: PageSize[] = new Array(pdfDoc.numPages);
      let next = 1;
      await Promise.all(Array.from({ length: Math.min(4, pdfDoc.numPages) }, async () => {
        while (!cancelled && next <= pdfDoc.numPages) {
          const number = next++;
          const page = await pdfDoc.getPage(number);
          const viewport = page.getViewport({ scale: 1 });
          pages[number - 1] = { width: viewport.width, height: viewport.height };
        }
      }));
      if (!cancelled) setLayout({ doc: pdfDoc, pages });
    })().catch((e) => { if (!cancelled) setError(String(e)); });
    return () => { cancelled = true; };
  }, [pdfDoc]);

  const ready = layout?.doc === pdfDoc;
  useEffect(() => {
    const container = scrollRef.current;
    if (!container) return;
    let resizeFrame = 0;
    const measure = () => {
      const width = container.clientWidth;
      setBounds({ width, height: container.clientHeight });
    };
    measure();
    const observer = new ResizeObserver(() => {
      cancelAnimationFrame(resizeFrame);
      resizeFrame = requestAnimationFrame(measure);
    });
    observer.observe(container);
    return () => { observer.disconnect(); cancelAnimationFrame(resizeFrame); };
  }, [pdfDoc, ready]);

  const currentSize = ready ? layout.pages[currentPage - 1] : undefined;
  const displayScale = currentSize && bounds.width ? fittedScale(currentSize, bounds, fitMode, scale) : null;
  useEffect(() => {
    if (displayScale !== null) useReaderStore.getState().setDisplayScale(displayScale);
  }, [displayScale]);

  useLayoutEffect(() => {
    if (!ready || !bounds.width) return;
    const container = scrollRef.current!;
    const elements = Array.from(container.querySelectorAll<HTMLElement>("[data-page]"));
    const pages = elements.map((element) => ({ top: element.offsetTop, height: element.offsetHeight }));
    if (!pages.length) return;
    const previous = geometryRef.current;
    let top: number;
    if (!previous || previous.doc !== pdfDoc || previous.navigation !== navigation) {
      top = (pages[useReaderStore.getState().currentPage - 1]?.top ?? 16) - 16;
    } else {
      // Preserve the same point within the page when the outline, pane size or zoom changes.
      const oldTop = positionRef.current.top + 16;
      let index = 0;
      while (index + 1 < previous.pages.length && previous.pages[index + 1].top <= oldTop) index++;
      const oldPage = previous.pages[index];
      const fraction = (oldTop - oldPage.top) / oldPage.height;
      top = pages[index].top + fraction * pages[index].height - 16;
    }
    geometryRef.current = { doc: pdfDoc!, navigation, pages };
    navigatingRef.current = true;
    container.scrollTo({ top, left: previous?.doc === pdfDoc ? positionRef.current.left : 0, behavior: "instant" });
    positionRef.current = { top: container.scrollTop, left: container.scrollLeft };
    // Scroll-originated page changes never trigger this effect.
    const frame = requestAnimationFrame(() => { navigatingRef.current = false; });
    return () => { cancelAnimationFrame(frame); navigatingRef.current = false; };
  }, [navigation, pdfDoc, ready, bounds.width, bounds.height, fitMode, scale]);

  useEffect(() => () => cancelAnimationFrame(frameRef.current), []);
  const handleScroll = useCallback(() => {
    const container = scrollRef.current;
    if (container) positionRef.current = { top: container.scrollTop, left: container.scrollLeft };
    if (frameRef.current) return;
    frameRef.current = requestAnimationFrame(() => {
      frameRef.current = 0;
      const container = scrollRef.current;
      if (!container || navigatingRef.current) return;
      const pages = container.querySelectorAll<HTMLElement>("[data-page]");
      const position = container.scrollTop + container.clientHeight * 0.25;
      let low = 0;
      let high = pages.length - 1;
      while (low < high) {
        const mid = Math.ceil((low + high) / 2);
        if (pages[mid].offsetTop <= position) low = mid;
        else high = mid - 1;
      }
      const store = useReaderStore.getState();
      if (pages.length && store.currentPage !== low + 1) store.updateCurrentPage(low + 1);
    });
  }, []);

  if (!pdfDoc) return (
    <div style={{ height: "100%", display: "flex", flexDirection: "column", alignItems: "center", justifyContent: "center", gap: 16 }}>
      <img src="/komodopdf_logo.svg" alt="Komodo PDF" style={{ width: 320, maxWidth: "80vw" }} />
      <p style={{ color: "var(--text-muted)" }}>Open a PDF to get started</p>
    </div>
  );

  return (
    <div ref={scrollRef} onScroll={handleScroll} aria-label="PDF pages" style={{ position: "relative", height: "100%",
      overflow: "auto", scrollbarGutter: "stable", background: "var(--canvas-bg)", padding: "16px", overflowAnchor: "none" }}>
      {error ? <p role="alert">Could not load PDF pages: {error}</p> : !ready ? <p>Loading pages…</p> :
        layout.pages.map((size, index) => {
          const renderScale = fittedScale(size, bounds, fitMode, scale);
          return <div key={index} data-page={index + 1} style={{ marginBottom: 16, width: "max-content", minWidth: "100%" }}>
            {bounds.width > 0 && scrollRef.current && <PdfPageView pdfDoc={pdfDoc} pageNum={index + 1}
              scale={renderScale} size={size} root={scrollRef.current} />}
          </div>;
        })}
    </div>
  );
}
