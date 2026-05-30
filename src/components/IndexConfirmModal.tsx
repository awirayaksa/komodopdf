import { useReaderStore } from "../state/useReaderStore";

export default function IndexConfirmModal() {
  const { setActiveModal, setPendingQuestion } = useReaderStore();

  return (
    <div style={modalOverlayStyle}>
      <div style={{ ...modalContentStyle, width: 400 }}>
        <h3 style={{ margin: "0 0 12px", fontSize: 16 }}>Index for AI</h3>
        <p
          style={{
            fontSize: 13,
            lineHeight: 1.5,
            color: "var(--text-muted)",
            marginBottom: 16,
          }}
        >
          Komodo will index this PDF once so it can answer with citations. Indexing runs
          <strong> locally on your device — free and private</strong> (default). This happens only
          the first time — after that you can ask freely.
        </p>
        <div style={{ display: "flex", gap: 8, justifyContent: "flex-end" }}>
          <button
            onClick={() => {
              setActiveModal("none");
              setPendingQuestion(null);
            }}
            style={secondaryBtnStyle}
          >
            Cancel
          </button>
          <button
            onClick={() => {
              setActiveModal("indexing");
            }}
            style={primaryBtnStyle}
          >
            Index now
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

const primaryBtnStyle: React.CSSProperties = {
  padding: "8px 16px",
  borderRadius: 6,
  backgroundColor: "var(--accent)",
  color: "#fff",
  fontWeight: 600,
  fontSize: 13,
};
