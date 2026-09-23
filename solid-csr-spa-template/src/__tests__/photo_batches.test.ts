import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { BatchStatusResponse } from "../generated";

const getBatchStatus = vi.fn();
vi.mock("../services/all_api", () => ({ photographyApi: { getBatchStatus } }));

const {
  MAX_POLL_BACKOFF_MS,
  POLL_INTERVAL_MS,
  nextPollDelay,
  resetBatchPollingForTests,
  setBatchCompletionHandler,
  trackBatch,
} = await import("../state/photo_batches");

let hidden = false;
Object.defineProperty(document, "hidden", {
  configurable: true,
  get: () => hidden,
});

const setHidden = (next: boolean) => {
  hidden = next;
  document.dispatchEvent(new Event("visibilitychange"));
};

const status = (done: boolean): BatchStatusResponse => ({
  batch_id: "batch-1",
  created_at: "2026-09-23T00:00:00Z",
  total: 1,
  completed: done ? 1 : 0,
  failed: 0,
  pending: done ? 0 : 1,
  done,
  items: [],
});

beforeEach(() => {
  vi.useFakeTimers();
  hidden = false;
  getBatchStatus.mockReset();
  resetBatchPollingForTests();
});

afterEach(() => {
  resetBatchPollingForTests();
  setBatchCompletionHandler(null);
  vi.useRealTimers();
});

describe("batch status polling", () => {
  it("caps exponential backoff", () => {
    expect(nextPollDelay(0)).toBe(POLL_INTERVAL_MS);
    expect(nextPollDelay(1)).toBe(POLL_INTERVAL_MS * 2);
    expect(nextPollDelay(3)).toBe(POLL_INTERVAL_MS * 8);
    expect(nextPollDelay(5)).toBe(MAX_POLL_BACKOFF_MS);
    expect(nextPollDelay(1_000)).toBe(MAX_POLL_BACKOFF_MS);
    expect(nextPollDelay(-1)).toBe(POLL_INTERVAL_MS);
  });

  it("backs off after server errors and returns to the base interval after success", async () => {
    getBatchStatus.mockRejectedValue({ status: 500 });
    trackBatch(status(false));

    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MS);
    expect(getBatchStatus).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MS * 2 - 1);
    expect(getBatchStatus).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(1);
    expect(getBatchStatus).toHaveBeenCalledTimes(2);

    getBatchStatus.mockResolvedValue({ data: status(false) });
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MS * 4);
    expect(getBatchStatus).toHaveBeenCalledTimes(3);
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MS);
    expect(getBatchStatus).toHaveBeenCalledTimes(4);
  });

  it("polls a newly tracked batch at the base interval even during a backoff", async () => {
    getBatchStatus.mockRejectedValue({ status: 503 });
    trackBatch(status(false));
    for (let round = 0; round < 4; round += 1) {
      await vi.advanceTimersByTimeAsync(nextPollDelay(round));
    }
    expect(getBatchStatus).toHaveBeenCalledTimes(4);

    getBatchStatus.mockResolvedValue({ data: status(false) });
    trackBatch({ ...status(false), batch_id: "batch-2" });
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MS);
    // One round polls both active batches.
    expect(getBatchStatus).toHaveBeenCalledTimes(6);
  });

  it("does not back off for missing batches inside the grace period", async () => {
    getBatchStatus.mockRejectedValue({ status: 404 });
    trackBatch(status(false));
    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MS * 3);
    expect(getBatchStatus).toHaveBeenCalledTimes(3);
  });

  it("pauses while the document is hidden and polls promptly when shown", async () => {
    getBatchStatus.mockResolvedValue({ data: status(false) });
    trackBatch(status(false));
    setHidden(true);
    await vi.advanceTimersByTimeAsync(MAX_POLL_BACKOFF_MS * 2);
    expect(getBatchStatus).not.toHaveBeenCalled();

    setHidden(false);
    await vi.advanceTimersByTimeAsync(0);
    expect(getBatchStatus).toHaveBeenCalledTimes(1);
  });

  it("stops polling once every batch is done and reports completion once", async () => {
    const onComplete = vi.fn();
    setBatchCompletionHandler(onComplete);
    getBatchStatus.mockResolvedValue({ data: status(true) });
    trackBatch(status(false));

    await vi.advanceTimersByTimeAsync(POLL_INTERVAL_MS);
    expect(getBatchStatus).toHaveBeenCalledTimes(1);
    expect(onComplete).toHaveBeenCalledTimes(1);
    await vi.advanceTimersByTimeAsync(MAX_POLL_BACKOFF_MS * 2);
    expect(getBatchStatus).toHaveBeenCalledTimes(1);
  });
});
