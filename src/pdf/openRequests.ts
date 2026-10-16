import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export function subscribeToPdfOpenRequests(
  open: (path: string) => Promise<void>,
  onError: (error: unknown) => void,
): () => void {
  let stopped = false;
  let draining = false;
  let requested = false;
  let unlisten: UnlistenFn | undefined;

  async function drain() {
    requested = true;
    if (draining || stopped) return;
    draining = true;
    try {
      while (requested && !stopped) {
        requested = false;
        const paths = await invoke<string[]>("take_pdf_open_requests");
        // Once removed from the native queue, finish the batch even if the
        // component unmounts. This also avoids losing requests in StrictMode.
        for (const path of paths) {
          try { await open(path); } catch (error) { onError(error); }
        }
        if (paths.length) requested = true;
      }
    } catch (error) {
      onError(error);
    } finally {
      draining = false;
    }
  }

  // Subscribe BEFORE draining: startup events may already have been emitted.
  void listen("pdf-open-requested", () => { void drain(); }).then((off) => {
    if (stopped) { off(); return; }
    unlisten = off;
    void drain();
  }).catch((error) => { if (!stopped) onError(error); });

  return () => {
    stopped = true;
    unlisten?.();
  };
}
