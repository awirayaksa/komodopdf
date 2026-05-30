interface DownloadProgress {
  doneBytes: number;
  totalBytes: number;
  file: string;
}

interface ModelDownloadModalProps {
  progress: DownloadProgress | null;
  onCancel: () => void;
}

export default function ModelDownloadModal({ progress, onCancel }: ModelDownloadModalProps) {
  const done = progress?.doneBytes ?? 0;
  const total = progress?.totalBytes ?? 1;
  const pct = total > 0 ? Math.min(100, Math.round((done / total) * 100)) : 0;
  const file = progress?.file ?? "";
  const doneMb = (done / 1024 / 1024).toFixed(1);
  const totalMb = (total / 1024 / 1024).toFixed(1);

  return (
    <div style={overlayStyle}>
      <div style={{ ...contentStyle, width: 360 }}>
        <h3 style={{ margin: "0 0 12px", fontSize: 16 }}>Downloading model...</h3>
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
            {doneMb} / {totalMb} MB — {file}
          </div>
        </div>
        <div style={{ display: "flex", justifyContent: "flex-end" }}>
          <button onClick={onCancel} style={secondaryBtnStyle}>
            Close
          </button>
        </div>
      </div>
    </div>
  );
}

const overlayStyle: React.CSSProperties = {
  position: "fixed",
  inset: 0,
  backgroundColor: "rgba(0,0,0,0.6)",
  display: "flex",
  alignItems: "center",
  justifyContent: "center",
  zIndex: 200,
};

const contentStyle: React.CSSProperties = {
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
