import { useReaderStore } from "../state/useReaderStore";

export default function AiPanelStub() {
  const { selection, fileName } = useReaderStore();

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        backgroundColor: "var(--surface-raised)",
        borderLeft: "1px solid var(--border)",
      }}
    >
      <div
        style={{
          padding: "12px 16px",
          borderBottom: "1px solid var(--border)",
          fontWeight: 600,
          fontSize: 14,
          display: "flex",
          alignItems: "center",
          gap: 8,
        }}
      >
        <span style={{ color: "var(--ai-accent)" }}>✦</span>
        <span>Ask Komodo</span>
      </div>

      <div style={{ flex: 1, padding: 16, overflow: "auto" }}>
        <div
          style={{
            padding: 16,
            borderRadius: 8,
            backgroundColor: "var(--surface)",
            border: "1px solid var(--border)",
            fontSize: 13,
            lineHeight: 1.5,
            color: "var(--text-muted)",
          }}
        >
          <p style={{ margin: "0 0 8px" }}>
            <strong style={{ color: "var(--text)" }}>AI grounding lands in Pass 2.</strong>
          </p>
          <p style={{ margin: 0 }}>
            For now, this panel is a preview. Once Pass 2 lands, you will be able to ask grounded
            questions about your PDF and click citations to jump to the source text.
          </p>
        </div>

        {selection && (
          <div style={{ marginTop: 16 }}>
            <div
              style={{
                fontSize: 11,
                color: "var(--text-muted)",
                textTransform: "uppercase",
                letterSpacing: 1,
                marginBottom: 8,
              }}
            >
              Current selection
            </div>
            <div
              style={{
                padding: 10,
                borderRadius: 6,
                backgroundColor: "var(--accent-tint)",
                color: "var(--surface)",
                fontSize: 13,
                lineHeight: 1.4,
                maxHeight: 120,
                overflow: "auto",
              }}
            >
              {selection}
            </div>
          </div>
        )}

        {fileName && (
          <div style={{ marginTop: 16 }}>
            <div
              style={{
                fontSize: 11,
                color: "var(--text-muted)",
                textTransform: "uppercase",
                letterSpacing: 1,
                marginBottom: 8,
              }}
            >
              Document
            </div>
            <div style={{ fontSize: 13, color: "var(--text)", wordBreak: "break-all" }}>
              {fileName}
            </div>
          </div>
        )}
      </div>

      <div
        style={{
          padding: "12px 16px",
          borderTop: "1px solid var(--border)",
          display: "flex",
          gap: 8,
        }}
      >
        <input
          type="text"
          placeholder="Ask anything..."
          disabled
          style={{
            flex: 1,
            padding: "8px 12px",
            borderRadius: 6,
            border: "1px solid var(--border)",
            backgroundColor: "var(--surface)",
            color: "var(--text-muted)",
            fontSize: 14,
            outline: "none",
          }}
        />
        <button
          disabled
          style={{
            padding: "8px 16px",
            borderRadius: 6,
            backgroundColor: "var(--ai-accent)",
            color: "var(--surface)",
            fontWeight: 600,
            opacity: 0.5,
          }}
        >
          Send
        </button>
      </div>
    </div>
  );
}
