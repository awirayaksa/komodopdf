import { useState, useRef, useEffect, useCallback } from "react";
import { invoke } from "@tauri-apps/api/core";
import { Channel } from "@tauri-apps/api/core";
import { useReaderStore } from "../state/useReaderStore";
import MarkdownRenderer from "./MarkdownRenderer";

interface ChatEvent {
  kind: "sources" | "token" | "done" | "error";
  data?: any;
}

export default function AiPanel() {
  const [input, setInput] = useState("");
  const scrollRef = useRef<HTMLDivElement>(null);
  const textareaRef = useRef<HTMLTextAreaElement>(null);

  function autoResize() {
    const ta = textareaRef.current;
    if (!ta) return;
    ta.style.height = "auto";
    const maxHeight = 120;
    if (ta.scrollHeight > maxHeight) {
      ta.style.height = maxHeight + "px";
      ta.style.overflowY = "auto";
    } else {
      ta.style.height = ta.scrollHeight + "px";
      ta.style.overflowY = "hidden";
    }
  }

  // Load config on mount
  useEffect(() => {
    (async () => {
      try {
        const settings: any = await invoke("get_settings");
        const store = useReaderStore.getState();
        const isConfigured = settings.configured === true;
        store.setApiKeyPresent(isConfigured);
        store.setConfigured(isConfigured);
        store.setConfig({
          embedBackend: settings.embedBackend === "remote" ? "remote" : "local",
          embedBaseUrl: settings.embedBaseUrl || "",
          embedModelRemote: settings.embedModelRemote || "",
          baseUrl: settings.chatBaseUrl || "https://api.openai.com/v1",
          chatModel: settings.chatModel || "gpt-4o-mini",
          embeddingModel: settings.embedModel || "nomic-embed-text-v1.5",
          topK: Number(settings.topK) || 6,
          persona: settings.persona || "Explain simply",
        });
      } catch {
        // ignore
      }
    })();
  }, []);

  const doAsk = useCallback(async (questionOverride?: string) => {
    const store = useReaderStore.getState();
    const q = questionOverride ?? input.trim();
    if (!q || store.streaming || !store.fileHash) return;

    if (!store.configured) {
      store.setPendingQuestion(q);
      store.setActiveModal("setup");
      setInput("");
      setTimeout(autoResize, 0);
      return;
    }

    if (store.indexState !== "ready") {
      store.setPendingQuestion(q);
      store.setActiveModal("indexConfirm");
      setInput("");
      setTimeout(autoResize, 0);
      return;
    }

    const messageCount = store.messages.length;
    store.addMessage({ role: "user", content: q });
    setInput("");
    setTimeout(autoResize, 0);
    store.setStreaming(true);

    // Include recent conversation history for multi-turn context (last 6 messages, excluding the current question)
    const history = store.messages.slice(-6).map((m) => ({
      role: m.role,
      content: m.content,
    }));

    const channel = new Channel<ChatEvent>();
    let sources: any[] = [];

    channel.onmessage = (event) => {
      const store2 = useReaderStore.getState();
      if (store2.pdfDoc !== store.pdfDoc) return;
      if (event.kind === "sources") {
        sources = event.data || [];
      } else if (event.kind === "token") {
        store2.appendToken(event.data as string);
      } else if (event.kind === "done") {
        store2.setStreaming(false);
        const msgs = [...store2.messages];
        const lastMsg = msgs[msgs.length - 1];
        if (lastMsg && lastMsg.role === "assistant") {
          const cited = parseCitations(lastMsg.content);
          lastMsg.citations = cited;
          lastMsg.sources = sources;
          msgs[msgs.length - 1] = lastMsg;
          useReaderStore.setState({ messages: msgs });
        }
      } else if (event.kind === "error") {
        store2.setStreaming(false);
        store2.appendToken(`\n[Error: ${event.data}]`);
      }
    };

    try {
      await invoke("ask", {
        fileHash: store.fileHash,
        question: q,
        currentPage: store.currentPage,
        selectionText: store.selection || null,
        selectionQuads: store.selectionQuads,
        channel,
        messageCount,
        history,
        persona: store.persona,
      });
    } catch (e: any) {
      const store2 = useReaderStore.getState();
      if (store2.pdfDoc !== store.pdfDoc) return;
      store2.setStreaming(false);
      store2.appendToken(`\n[Error: ${e.toString()}]`);
    }
  }, [input]);

  const doAskRef = useRef(doAsk);
  doAskRef.current = doAsk;

  // Auto-send pending question after indexing completes
  useEffect(() => {
    return useReaderStore.subscribe((state, prevState) => {
      if (
        state.indexState === "ready" &&
        prevState.indexState !== "ready" &&
        state.pendingQuestion
      ) {
        const q = state.pendingQuestion;
        useReaderStore.getState().setPendingQuestion(null);
        setTimeout(() => doAskRef.current(q), 50);
      }
    });
  }, []);

  function parseCitations(text: string): number[] {
    const matches = text.match(/\[(\d+)\]/g);
    if (!matches) return [];
    const nums = matches.map((m) => parseInt(m.slice(1, -1), 10));
    return [...new Set(nums)].sort((a, b) => a - b);
  }

  function handleCitationClick(sourceIdx: number, message: typeof messages[number]) {
    const store = useReaderStore.getState();
    const source = message.sources?.find((s) => s.idx === sourceIdx);
    if (!source) return;

    store.setCurrentPage(source.page);
    store.setActiveHighlight({ page: source.page, quads: source.quads });
  }

  function handleAskAboutThis() {
    const store = useReaderStore.getState();
    if (!store.selection) return;
    const prompt = `Explain this: "${store.selection.slice(0, 200)}${
      store.selection.length > 200 ? "..." : ""
    }"`;
    doAsk(prompt);
  }

  async function handleReindex() {
    if (!fileHash) return;
    const confirmed = window.confirm(
      "Reindex this document?\n\nThis will delete the existing index and rebuild it with the current embedding model. This is useful if you changed the embedding model in Settings."
    );
    if (!confirmed) return;

    try {
      await invoke("delete_document", { fileHash });
      setIndexState("idle");
      setActiveModal("indexing");
    } catch (e: any) {
      alert("Failed to clear old index: " + e.toString());
    }
  }

  const {
    selection,
    fileName,
    fileHash,
    messages,
    streaming,
    configured,
    indexState,
    setActiveModal,
    setIndexState,
  } = useReaderStore();

  useEffect(() => {
    if (scrollRef.current) {
      scrollRef.current.scrollTop = scrollRef.current.scrollHeight;
    }
  }, [messages, streaming]);

  return (
    <div
      style={{
        display: "flex",
        flexDirection: "column",
        height: "100%",
        overflow: "hidden",
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
          justifyContent: "space-between",
        }}
      >
        <div style={{ display: "flex", alignItems: "center", gap: 8 }}>
          <span style={{ color: "var(--ai-accent)" }}>✦</span>
          <span>Ask Komodo</span>
        </div>
        <button
          onClick={() => setActiveModal("setup")}
          title="Settings"
          style={{
            padding: "4px 8px",
            borderRadius: 6,
            fontSize: 16,
            background: "var(--surface)",
            border: "1px solid var(--border)",
          }}
        >
          ⚙
        </button>
      </div>

      <div ref={scrollRef} style={{ flex: 1, padding: 16, overflow: "auto" }}>
        {messages.length === 0 && !fileName && (
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
              <strong style={{ color: "var(--text)" }}>
                Ask questions about your PDF.
              </strong>
            </p>
            <p style={{ margin: 0 }}>
              Open a document, then type a question here. Komodo will ground its answer in the
              text and show clickable page citations.
            </p>
          </div>
        )}

        {messages.map((msg, i) => (
          <div key={i} style={{ marginBottom: 16 }}>
            <div
              style={{
                fontSize: 11,
                color: "var(--text-muted)",
                textTransform: "uppercase",
                letterSpacing: 1,
                marginBottom: 4,
              }}
            >
              {msg.role === "user" ? "You" : "Komodo"}
            </div>
            <div
              style={{
                padding: 10,
                borderRadius: 8,
                backgroundColor:
                  msg.role === "user" ? "var(--surface)" : "var(--surface-lowered)",
                border: "1px solid var(--border)",
                fontSize: 13,
                lineHeight: 1.5,
                whiteSpace: msg.role === "user" ? "pre-wrap" : undefined,
              }}
            >
              {msg.role === "assistant" ? (
                <MarkdownRenderer content={msg.content} />
              ) : (
                msg.content
              )}
            </div>
            {msg.role === "assistant" &&
              msg.citations &&
              msg.citations.length > 0 && (
                <div
                  style={{
                    display: "flex",
                    flexWrap: "wrap",
                    gap: 6,
                    marginTop: 8,
                  }}
                >
                  {msg.citations.map((c) => (
                    <button
                      key={c}
                      onClick={() => handleCitationClick(c, msg)}
                      style={{
                        padding: "4px 10px",
                        borderRadius: 12,
                        backgroundColor: "rgba(245, 158, 11, 0.15)",
                        border: "1px solid rgba(245, 158, 11, 0.4)",
                        color: "var(--ai-accent)",
                        fontSize: 12,
                        fontWeight: 500,
                        cursor: "pointer",
                      }}
                    >
                      p.
                      {msg.sources?.find((s) => s.idx === c)?.page ?? c}
                    </button>
                  ))}
                </div>
              )}
          </div>
        ))}

        {streaming && (
          <div style={{ display: "flex", gap: 6, alignItems: "center" }}>
            <div
              style={{
                width: 6,
                height: 6,
                borderRadius: "50%",
                backgroundColor: "var(--ai-accent)",
                animation: "pulse 1.2s infinite ease-in-out",
              }}
            />
            <span style={{ fontSize: 12, color: "var(--text-muted)" }}>
              Thinking...
            </span>
          </div>
        )}
      </div>

      {selection && (
        <div style={{ padding: "0 16px 8px" }}>
          <button
            onClick={handleAskAboutThis}
            style={{
              width: "100%",
              padding: "8px 12px",
              borderRadius: 6,
              backgroundColor: "var(--accent-tint)",
              border: "1px solid var(--border)",
              color: "var(--accent-hover)",
              fontSize: 12,
              fontWeight: 500,
            }}
          >
            Ask about this selection
          </button>
        </div>
      )}

      <div
        style={{
          padding: "12px 16px",
          borderTop: "1px solid var(--border)",
          display: "flex",
          flexDirection: "column",
          gap: 8,
        }}
      >
        <div style={{ display: "flex", gap: 8 }}>
          <textarea
            ref={textareaRef}
            value={input}
            onChange={(e) => {
              setInput(e.target.value);
              autoResize();
            }}
            onKeyDown={(e) => {
              if (e.key === "Enter" && !e.shiftKey) {
                if (e.nativeEvent.isComposing || e.keyCode === 229) return;
                e.preventDefault();
                doAsk();
              }
            }}
            placeholder={
              configured ? "Ask anything..." : "Configure API key to ask..."
            }
            disabled={!configured || streaming || !fileName}
            rows={1}
            style={{
              flex: 1,
              padding: "8px 12px",
              minWidth: 0,
              borderRadius: 6,
              border: "1px solid var(--border)",
              backgroundColor: "var(--surface)",
              color: "var(--text)",
              fontSize: 14,
              outline: "none",
              fontFamily: "inherit",
              resize: "none",
              overflowY: "hidden",
              minHeight: 38,
              maxHeight: 120,
              lineHeight: 1.4,
            }}
          />
          <button
            onClick={() => doAsk()}
            disabled={!configured || streaming || !input.trim() || !fileName}
            style={{
              padding: "8px 16px",
              borderRadius: 6,
              backgroundColor: "var(--ai-accent)",
              color: "#fff",
              fontWeight: 600,
              fontSize: 13,
              opacity:
                !configured || streaming || !input.trim() || !fileName ? 0.5 : 1,
              alignSelf: "flex-end",
            }}
          >
            Send
          </button>
        </div>
        <div
          style={{
            fontSize: 11,
            color: "var(--text-muted)",
            textAlign: "center",
            display: "flex",
            alignItems: "center",
            justifyContent: "center",
            gap: 8,
          }}
        >
          {!configured
            ? "Not configured — open Settings (⚙) to add your API key"
            : indexState === "ready"
            ? "Ready to answer"
            : indexState === "indexing"
            ? "Indexing..."
            : "Not indexed — first ask will index this document"}
          {configured && indexState === "ready" && fileHash && (
            <button
              onClick={handleReindex}
              title="Delete existing index and rebuild with current embedding model"
              style={{
                fontSize: 11,
                color: "var(--ai-accent)",
                background: "none",
                border: "none",
                cursor: "pointer",
                textDecoration: "underline",
                padding: 0,
              }}
            >
              Reindex
            </button>
          )}
        </div>
      </div>
    </div>
  );
}
