import { beforeEach, expect, it, vi } from "vitest";
import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { subscribeToPdfOpenRequests } from "../../src/pdf/openRequests";

vi.mock("@tauri-apps/api/core", () => ({ invoke: vi.fn() }));
vi.mock("@tauri-apps/api/event", () => ({ listen: vi.fn() }));

let wake: () => void;
let queue: string[];
const off = vi.fn();
const flush = () => new Promise<void>((resolve) => setTimeout(resolve, 0));

beforeEach(() => {
  vi.resetAllMocks();
  queue = [];
  vi.mocked(listen).mockImplementation(async (_event, callback) => {
    wake = () => callback({} as never);
    return off;
  });
  vi.mocked(invoke).mockImplementation(async () => queue.splice(0) as never);
});

it("drains cold-start paths after subscribing, even when the startup event was missed", async () => {
  queue.push("C:\\日本語 資料\\100%20 #1.pdf");
  const open = vi.fn(async () => {});
  const stop = subscribeToPdfOpenRequests(open, vi.fn());
  await flush();
  expect(listen).toHaveBeenCalledWith("pdf-open-requested", expect.any(Function));
  expect(invoke).toHaveBeenCalledWith("take_pdf_open_requests");
  expect(open.mock.calls).toEqual([["C:\\日本語 資料\\100%20 #1.pdf"]]);
  stop();
  expect(off).toHaveBeenCalledOnce();
});

it("processes repeated launches and multiple files serially, including arrivals while loading", async () => {
  let finish!: () => void;
  const open = vi.fn((path: string) => path === "first.pdf"
    ? new Promise<void>((resolve) => { finish = resolve; }) : Promise.resolve());
  queue.push("first.pdf", "second.pdf");
  const stop = subscribeToPdfOpenRequests(open, vi.fn());
  await flush();
  queue.push("third.pdf");
  wake(); wake();
  expect(open.mock.calls).toEqual([["first.pdf"]]);
  finish();
  await flush();
  expect(open.mock.calls).toEqual([["first.pdf"], ["second.pdf"], ["third.pdf"]]);
  stop();
});

it("continues after an unreadable PDF and consumes later requests", async () => {
  queue.push("missing.pdf", "valid.pdf");
  const open = vi.fn().mockRejectedValueOnce(new Error("missing")).mockResolvedValue(undefined);
  const error = vi.fn();
  const stop = subscribeToPdfOpenRequests(open, error);
  await flush();
  expect(error).toHaveBeenCalledOnce();
  expect(open.mock.calls).toEqual([["missing.pdf"], ["valid.pdf"]]);
  stop();
});

it("cleans up a pending subscription without draining twice in React StrictMode", async () => {
  let subscribed!: (off: () => void) => void;
  vi.mocked(listen).mockImplementationOnce(() => new Promise((resolve) => { subscribed = resolve; }));
  queue.push("startup.pdf");
  const open = vi.fn(async () => {});
  const firstStop = subscribeToPdfOpenRequests(open, vi.fn());
  firstStop();
  const staleOff = vi.fn();
  subscribed(staleOff);
  const stop = subscribeToPdfOpenRequests(open, vi.fn());
  await flush();
  expect(staleOff).toHaveBeenCalledOnce();
  expect(open.mock.calls).toEqual([["startup.pdf"]]);
  stop();
});

it("finishes a batch already removed from Rust when cleanup races with the command", async () => {
  let reply!: (paths: string[]) => void;
  vi.mocked(invoke).mockImplementationOnce(() => new Promise((resolve) => { reply = resolve as typeof reply; }));
  const open = vi.fn(async () => {});
  const stop = subscribeToPdfOpenRequests(open, vi.fn());
  await flush();
  stop();
  reply(["claimed.pdf"]);
  await flush();
  expect(open.mock.calls).toEqual([["claimed.pdf"]]);
});
