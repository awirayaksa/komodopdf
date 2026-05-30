import { useEffect, useRef, useCallback, useState } from "react";
import { useReaderStore } from "../state/useReaderStore";
import { renderPageToCanvas } from "../pdf/pageRender";
import { buildTextLayer } from "../pdf/textLayer";
import type { Quad } from "../state/useReaderStore";

function transformQuad(quad: Quad, renderScale: number): { x: number; y: number; w: number; h: number } {
  const x = Math.min(quad.x0, quad.x1) * renderScale;
  const y = Math.min(quad.y0, quad.y1) * renderScale;
  const w = Math.abs(quad.x1 - quad.x0) * renderScale;
  const h = Math.abs(quad.y1 - quad.y0) * renderScale;
  return { x, y, w, h };
}

function PdfPageView({ pageNum }: { pageNum: number }) {
  const { pdfDoc, scale, fitMode, setHasText, activeHighlight } = useReaderStore();
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const textRef = useRef<HTMLDivElement>(null);
  const wrapperRef = useRef<HTMLDivElement>(null);
  const [isVisible, setIsVisible] = useState(false);
  const [rendered, setRendered] = useState(false);
  const [hasText, setLocalHasText] = useState<boolean | null>(null);
  const [renderScale, setRenderScale] = useState(1);
  const [flashKey, setFlashKey] = useState(0);
  const [placeholderSize, setPlaceholderSize] = useState<{ width: number; height: number } | null>(null);

  useEffect(() => {
    const el = wrapperRef.current;
    if (!el) return;
    const observer = new IntersectionObserver(
      ([entry]) => setIsVisible(entry.isIntersecting),
      { root: el.parentElement?.parentElement ?? null, threshold: 0 }
    );
    observer.observe(el);
    return () => observer.disconnect();
  }, []);

  const doRender = useCallback(async () => {
    if (!pdfDoc || !canvasRef.current || !textRef.current) return;
    const page = await pdfDoc.getPage(pageNum);
    let rs = scale;
    if (fitMode === "width" && wrapperRef.current) {
      const cw = wrapperRef.current.clientWidth - 24;
      rs = cw / page.getViewport({ scale: 1 }).width;
    } else if (fitMode === "page" && wrapperRef.current) {
      const cw = wrapperRef.current.clientWidth - 24;
      const ch = wrapperRef.current.clientHeight || 800;
      const v1 = page.getViewport({ scale: 1 });
      rs = Math.min(cw / v1.width, ch / v1.height);
    }
    await renderPageToCanvas(page, canvasRef.current, rs);
    const hasTextResult = await buildTextLayer(page, textRef.current, rs);
    setLocalHasText(hasTextResult);
    setHasText(pageNum, hasTextResult);
    setRenderScale(rs);
    setRendered(true);
  }, [pdfDoc, pageNum, scale, fitMode, setHasText]);

  useEffect(() => {
    if (isVisible && !rendered) {
      doRender();
    }
  }, [isVisible, rendered, doRender]);

  useEffect(() => {
    if (!rendered) return;
    doRender();
  }, [scale, fitMode, doRender]);

  useEffect(() => {
    if (activeHighlight?.page === pageNum) {
      setFlashKey((k) => k + 1);
      const timer = setTimeout(() => {
        useReaderStore.getState().clearHighlight();
      }, 3000);
      return () => clearTimeout(timer);
    }
  }, [activeHighlight, pageNum]);

  useEffect(() => {
    if (!pdfDoc) return;
    let cancelled = false;
    pdfDoc.getPage(pageNum).then((page) => {
      if (cancelled) return;
      const v1 = page.getViewport({ scale: 1 });
      let rs = scale;
      if (fitMode) {
        const parentWidth = wrapperRef.current?.parentElement?.clientWidth;
        if (parentWidth) {
          const cw = Math.max(1, parentWidth - 24);
          if (fitMode === "width") {
            rs = cw / v1.width;
          } else if (fitMode === "page") {
            const parentHeight = wrapperRef.current?.parentElement?.clientHeight || 800;
            rs = Math.min(cw / v1.width, parentHeight / v1.height);
          }
        }
      }
      const viewport = page.getViewport({ scale: rs });
      setPlaceholderSize({ width: Math.floor(viewport.width), height: Math.floor(viewport.height) });
    });
    return () => {
      cancelled = true;
    };
  }, [pdfDoc, pageNum, scale, fitMode]);

  const showHighlight = activeHighlight?.page === pageNum;

  return (
      <div
        ref={wrapperRef}
        style={{
          position: "relative",
          margin: "0",
          boxShadow: "0 2px 8px rgba(0,0,0,0.2)",
          backgroundColor: "var(--page-bg)",
          display: "inline-block",
          maxWidth: "100%",
          minHeight: placeholderSize?.height,
        }}
      >
      <canvas ref={canvasRef} style={{ display: "block" }} />
      <div
        ref={textRef}
        style={{
          position: "absolute",
          top: 0,
          left: 0,
          right: 0,
          bottom: 0,
          overflow: "hidden",
        }}
        onMouseUp={() => {
          const sel = window.getSelection()?.toString() || "";
          useReaderStore.getState().setSelection(sel);
        }}
      />
      {showHighlight &&
        activeHighlight?.quads.map((quad, i) => {
          const r = transformQuad(quad, renderScale);
          return (
            <div
              key={`${flashKey}-${i}`}
              style={{
                position: "absolute",
                left: r.x,
                top: r.y,
                width: r.w,
                height: r.h,
                backgroundColor: "rgba(245, 158, 11, 0.35)",
                border: "1px solid rgba(245, 158, 11, 0.8)",
                borderRadius: 2,
                pointerEvents: "none",
                animation: "flash-amber 3s ease-out forwards",
              }}
            />
          );
        })}
      {hasText === false && (
        <div
          style={{
            position: "absolute",
            bottom: 8,
            left: "50%",
            transform: "translateX(-50%)",
            backgroundColor: "var(--badge-bg)",
            color: "var(--text-muted)",
            padding: "4px 12px",
            borderRadius: 12,
            fontSize: 12,
            border: "1px solid var(--border)",
            pointerEvents: "none",
          }}
        >
          No text layer — AI unavailable here
        </div>
      )}
      <div
        style={{
          position: "absolute",
          top: 4,
          right: 8,
          fontSize: 11,
          color: "var(--text-muted)",
          pointerEvents: "none",
        }}
      >
        {pageNum}
      </div>
    </div>
  );
}

export default function ReaderPane() {
  const { totalPages, currentPage, pdfDoc } = useReaderStore();
  const scrollRef = useRef<HTMLDivElement>(null);
  const suppressScrollRef = useRef(false);
  const scrollTimeoutRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    if (!scrollRef.current || !pdfDoc) return;
    const children = scrollRef.current.querySelectorAll("[data-page]");
    const target = children[currentPage - 1] as HTMLElement | undefined;
    if (target) {
      suppressScrollRef.current = true;
      if (scrollTimeoutRef.current) clearTimeout(scrollTimeoutRef.current);
      const container = scrollRef.current;
      const containerRect = container.getBoundingClientRect();
      const targetRect = target.getBoundingClientRect();
      const scrollTop = container.scrollTop + (targetRect.top - containerRect.top) - container.clientTop;
      container.scrollTo({ top: scrollTop, behavior: "smooth" });
      scrollTimeoutRef.current = setTimeout(() => {
        suppressScrollRef.current = false;
      }, 800);
    }
    return () => {
      if (scrollTimeoutRef.current) clearTimeout(scrollTimeoutRef.current);
    };
  }, [currentPage, pdfDoc]);

  const handleScroll = useCallback(() => {
    if (!scrollRef.current) return;
    if (suppressScrollRef.current) return;
    const children = scrollRef.current.querySelectorAll("[data-page]");
    let closest = 1;
    let minDist = Infinity;
    children.forEach((child, idx) => {
      const el = child as HTMLElement;
      const rect = el.getBoundingClientRect();
      const dist = Math.abs(rect.top);
      if (dist < minDist) {
        minDist = dist;
        closest = idx + 1;
      }
    });
    const state = useReaderStore.getState();
    if (state.currentPage !== closest) {
      state.setCurrentPage(closest);
    }
  }, []);

  if (!pdfDoc) {
    return (
      <div
        style={{
          height: "100%",
          display: "flex",
          flexDirection: "column",
          alignItems: "center",
          justifyContent: "center",
          gap: 16,
          backgroundColor: "var(--surface)",
          overflow: "hidden",
        }}
      >
        <img
          src="/komodopdf_logo.svg"
          alt="Komodo PDF"
          style={{ width: 320, maxWidth: "80vw", opacity: 0.9 }}
        />
        <p style={{ color: "var(--text-muted)", fontSize: 13, letterSpacing: 2, textTransform: "uppercase" }}>
          READ SMARTER · ASK ANYTHING
        </p>
        <p style={{ color: "var(--text-muted)", fontSize: 14, marginTop: -8 }}>
          Open a PDF to get started
        </p>
      </div>
    );
  }

  return (
    <div
      ref={scrollRef}
      onScroll={handleScroll}
      style={{
        height: "100%",
        overflow: "auto",
        backgroundColor: "var(--canvas-bg)",
        textAlign: "center",
        padding: "0 16px 40px",
      }}
    >
      {Array.from({ length: totalPages }, (_, i) => (
        <div key={i + 1} data-page={i + 1} style={{ margin: "16px 0" }}>
          <PdfPageView pageNum={i + 1} />
        </div>
      ))}
    </div>
  );
}
