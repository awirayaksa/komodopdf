import { useReaderStore } from "../state/useReaderStore";

export default function IndexingModal() {
  const { indexProgress, setActiveModal } = useReaderStore();
  const done = indexProgress?.done ?? 0;
  const total = indexProgress?.total ?? 1;
  const pct = Math.min(100, Math.round((done / total) * 100));

  return (
    <div style={modalOverlayStyle}>
      <div style={{ ...modalContentStyle, width: 360 }}>
        <h3 style={{ margin: "0 0 12px", fontSize: 16 }}>Indexing...</h3>
        <div style={{ marginBottom: 12 }}>
          <div
            style={{
              height: 8,
              backgroundColor: "var(--surface)",
              borderRadius: 4,
              overflow: "hidden",
            }}
          >
            <div
              style={{
                height: "100%",
                width: `${pct}%`,
                backgroundColor: "var(--ai-accent)",
                transition: "width 0.3s ease",
              }}
            />
          </div>
          <div
            style={{
              fontSize: 12,
              color: "var(--text-muted)",
              marginTop: 6,
              textAlign: "center",
            }}
          >
            {done} / {total} pages
          </div>
        </div>
        <div style={{ display: "flex", justifyContent: "flex-end" }}>
          <button onClick={() => setActiveModal("none")} style={secondaryBtnStyle}>
            Cancel
          </button>
        </div>
      </div>
    </div>
  );
}

const modalOverlayStyle: React.CSSProperties = {
  position: "fixed",
  inset: 0,
  backgroundColor: "rgba(0,0,0,0.6)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  zIndex: 100,
};

const modalContentStyle: React.CSSProperties = {
  backgroundColor: "var(--surface-raised)",
  border: "1px solid var(--border)",
  borderRadius: 10,
  padding: 24,
  maxWidth: "90vw",
  maxHeight: "90vh",
  overflow: "auto",
};

const secondaryBtnStyle: React.CSSProperties = {
  padding: "8px 14px",
  borderRadius: 6,
  backgroundColor: "var(--surface)",
  border: "1px solid var(--border)",
  color: "var(--text)",
  fontSize: 13,
};
