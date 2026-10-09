import { useState } from "react";
import { useReaderStore } from "../state/useReaderStore";

interface OutlineNodeProps {
  items: any[];
  depth?: number;
}

function OutlineNode({ items, depth = 0 }: OutlineNodeProps) {
  const { pdfDoc, setCurrentPage } = useReaderStore();
  const [expanded, setExpanded] = useState<Set<number>>(new Set());

  async function navigateTo(dest: any) {
    if (!pdfDoc) return;
    try {
      let pageRef: any;
      if (typeof dest === "string") {
        const explicitDest = await pdfDoc.getDestination(dest);
        if (!explicitDest) return;
        pageRef = explicitDest[0];
      } else if (Array.isArray(dest) && dest.length > 0) {
        pageRef = dest[0];
      } else {
        return;
      }
      const pageIndex = typeof pageRef === "number" ? pageRef : await pdfDoc.getPageIndex(pageRef);
      setCurrentPage(pageIndex + 1);
    } catch {
      // ignore
    }
  }

  return (
    <ul style={{ listStyle: "none", margin: 0, padding: 0, paddingLeft: depth > 0 ? 12 : 0 }}>
      {items.map((item, i) => {
        const hasChildren = item.items && item.items.length > 0;
        const isExpanded = expanded.has(i);
        return (
          <li key={`${depth}-${i}`}>
            <button
              onClick={() => {
                if (item.dest) navigateTo(item.dest);
                if (hasChildren) {
                  setExpanded((prev) => {
                    const next = new Set(prev);
                    if (next.has(i)) next.delete(i);
                    else next.add(i);
                    return next;
                  });
                }
              }}
              style={{
                display: "flex",
                alignItems: "center",
                gap: 4,
                width: "100%",
                textAlign: "left",
                padding: "4px 8px",
                borderRadius: 4,
                fontSize: 13,
                color: "var(--text)",
                backgroundColor: "transparent",
                border: "none",
                cursor: "pointer",
                whiteSpace: "nowrap",
                overflow: "hidden",
                textOverflow: "ellipsis",
                transition: "background-color 0.1s, color 0.1s",
              }}
              onMouseEnter={(e) => {
                (e.currentTarget as HTMLButtonElement).style.backgroundColor = "var(--accent-tint)";
                (e.currentTarget as HTMLButtonElement).style.color = "var(--accent-hover)";
              }}
              onMouseLeave={(e) => {
                (e.currentTarget as HTMLButtonElement).style.backgroundColor = "transparent";
                (e.currentTarget as HTMLButtonElement).style.color = "var(--text)";
              }}
            >
              {hasChildren && (
                <span style={{ fontSize: 10, flexShrink: 0 }}>
                  {isExpanded ? "▼" : "▶"}
                </span>
              )}
              <span style={{ overflow: "hidden", textOverflow: "ellipsis" }}>
                {item.title}
              </span>
            </button>
            {hasChildren && isExpanded && (
              <OutlineNode items={item.items} depth={depth + 1} />
            )}
          </li>
        );
      })}
    </ul>
  );
}

export default function OutlineSidebar() {
  const { outline, outlineOpen, toggleOutline } = useReaderStore();

  if (!outlineOpen) {
    return (
      <button
        onClick={toggleOutline}
        title="Show outline"
        style={{
          width: 28,
          flexShrink: 0,
          alignSelf: "stretch",
          padding: 0,
          backgroundColor: "var(--surface-raised)",
          borderRight: "1px solid var(--border)",
          display: "flex",
          alignItems: "center",
          justifyContent: "center",
        }}
      >
        ☰
      </button>
    );
  }

  return (
    <aside
      style={{
        width: 260,
        flexShrink: 0,
        backgroundColor: "var(--surface-raised)",
        borderRight: "1px solid var(--border)",
        display: "flex",
        flexDirection: "column",
        overflow: "hidden",
      }}
    >
      <div
        style={{
          display: "flex",
          alignItems: "center",
          justifyContent: "space-between",
          padding: "8px 12px",
          borderBottom: "1px solid var(--border)",
          fontSize: 13,
          fontWeight: 600,
          color: "var(--text-muted)",
          textTransform: "uppercase",
          letterSpacing: 1,
        }}
      >
        <span>Outline</span>
        <button
          onClick={toggleOutline}
          title="Hide outline"
          style={{ padding: "2px 6px", borderRadius: 4, fontSize: 12 }}
        >
          ←
        </button>
      </div>
      <div style={{ flex: 1, overflow: "auto", padding: "8px" }}>
        {outline.length === 0 ? (
          <div style={{ padding: 12, fontSize: 13, color: "var(--text-muted)" }}>
            No outline available
          </div>
        ) : (
          <OutlineNode items={outline} />
        )}
      </div>
    </aside>
  );
}
