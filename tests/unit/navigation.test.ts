import { beforeEach, expect, it } from "vitest";
import { useReaderStore } from "../../src/state/useReaderStore";

beforeEach(() => useReaderStore.setState({ totalPages: 12, currentPage: 1, navigationRequest: { page: 1, id: 0 } }));

it("updates the visible page without requesting another scroll", () => {
  const request = useReaderStore.getState().navigationRequest;
  useReaderStore.getState().updateCurrentPage(4);
  expect(useReaderStore.getState().currentPage).toBe(4);
  expect(useReaderStore.getState().navigationRequest).toBe(request);
});

it("requests navigation for toolbar, outline, and repeated same-page citations", () => {
  useReaderStore.getState().nextPage();
  expect(useReaderStore.getState().navigationRequest).toEqual({ page: 2, id: 1 });
  useReaderStore.getState().setCurrentPage(2);
  expect(useReaderStore.getState().navigationRequest).toEqual({ page: 2, id: 2 });
  useReaderStore.getState().setCurrentPage(100);
  expect(useReaderStore.getState().navigationRequest.page).toBe(12);
});

it("requires reindexing after changing embeddings but preserves it when only chat changes", () => {
  useReaderStore.setState({ indexState: "ready", embedBackend: "local", embeddingModel: "bge-small-en-v1.5" });
  useReaderStore.getState().setConfig({ chatModel: "another-chat-model" });
  expect(useReaderStore.getState().indexState).toBe("ready");
  useReaderStore.getState().setConfig({ embeddingModel: "multilingual-e5-small" });
  expect(useReaderStore.getState().indexState).toBe("idle");
});
