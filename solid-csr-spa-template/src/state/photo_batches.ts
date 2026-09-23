// Module-scoped global store + poller for photograph batch-upload sessions.
//
// Survives route navigation (module singleton, like state/auth.ts). A single
// shared timer polls all active batches with an in-flight guard; it stops when
// nothing is active and restarts on trackBatch. Rounds with server or network
// errors back off exponentially up to a cap, and polling pauses while the
// document is hidden, resuming with an immediate poll when it is shown again.
// A 404 from the poller marks a batch _missing (after a short grace) rather
// than triggering logout, because the status endpoint returns 404 (never 403)
// for absent/not-owned batches.

import { createStore, flush } from "solid-js";
import { photographyApi } from "../services/all_api";
import type {
  BatchStatusResponse,
  BatchUploadResponse,
} from "../generated";

export interface BatchEntry {
  batch_id: string;
  status: BatchStatusResponse | null;
  _missing: boolean;
  _firstSeen: number;
  _gridRefreshed: boolean;
}

const [batches, setBatches] = createStore<Record<string, BatchEntry>>({});
export { batches };

export const POLL_INTERVAL_MS = 1200;
export const MAX_POLL_BACKOFF_MS = 30_000;
const MISSING_GRACE_MS = 6000;

let pollTimer: ReturnType<typeof setTimeout> | null = null;
let inFlight = false;
let consecutiveFailedRounds = 0;
let visibilityListening = false;
let completionHandler: (() => void) | null = null;

/** Delay before the next poll after `failedRounds` consecutive error rounds. */
export function nextPollDelay(failedRounds: number): number {
  // Clamp the exponent so large streaks cannot overflow to Infinity.
  const exponent = Math.min(Math.max(failedRounds, 0), 16);
  return Math.min(POLL_INTERVAL_MS * 2 ** exponent, MAX_POLL_BACKOFF_MS);
}

/**
 * Register a page-level callback fired once when any tracked batch reaches
 * `done`. The photographs page uses it to refresh the grid. Pass null on cleanup
 * to avoid a stale closure over an unmounted page.
 */
export function setBatchCompletionHandler(handler: (() => void) | null): void {
  completionHandler = handler;
}

/** Number of batches still being processed (reactive). */
export function activeBatchCount(): number {
  let count = 0;
  for (const id of Object.keys(batches)) {
    const entry = batches[id];
    if (entry && !entry._missing && !entry.status?.done) count += 1;
  }
  return count;
}

function isActive(entry: BatchEntry): boolean {
  return !entry._missing && !entry.status?.done;
}

/** Begin tracking a batch from an existing status snapshot. */
export function trackBatch(initial: BatchStatusResponse): void {
  setBatches((state) => {
    state[initial.batch_id] = {
      batch_id: initial.batch_id,
      status: initial,
      _missing: false,
      _firstSeen: Date.now(),
      _gridRefreshed: false,
    };
  });
  // Writes land on the microtask flush; the active-batch scan must see this one.
  flush();
  // A fresh upload was just accepted, so the server is reachable again.
  consecutiveFailedRounds = 0;
  ensurePolling();
}

/** Begin tracking from the immediate batch-upload acceptance response. */
export function trackFromUpload(resp: BatchUploadResponse): void {
  const nowIso = new Date().toISOString();
  const synthetic: BatchStatusResponse = {
    batch_id: resp.batch_id,
    created_at: nowIso,
    total: resp.total,
    completed: 0,
    failed: 0,
    pending: resp.total,
    done: false,
    items: resp.items.map((item) => ({
      item_id: item.item_id,
      file_name: item.file_name,
      original_size_bytes: 0,
      status: { status: "queued" as const },
      created_at: nowIso,
      updated_at: nowIso,
    })),
  };
  trackBatch(synthetic);
}

/** Drop finished (done) and missing batches from the tracker. */
export function clearFinishedBatches(): void {
  setBatches((state) => {
    for (const id of Object.keys(state)) {
      const entry = state[id];
      if (entry && (entry._missing || entry.status?.done)) {
        delete state[id];
      }
    }
  });
  // Writes land on the microtask flush; force them through so the
  // active-batch scan below sees the post-clear state.
  flush();
  maybeStopPolling();
}

function hasActiveBatches(): boolean {
  for (const id of Object.keys(batches)) {
    const entry = batches[id];
    if (entry && isActive(entry)) return true;
  }
  return false;
}

function documentHidden(): boolean {
  return typeof document !== "undefined" && document.hidden;
}

function handleVisibilityChange(): void {
  if (documentHidden()) {
    clearPollTimer();
    return;
  }
  // Show progress promptly on return; an error streak keeps its backoff for
  // the polls that follow.
  schedulePoll(0);
}

function setVisibilityListening(listen: boolean): void {
  if (typeof document === "undefined" || listen === visibilityListening) return;
  visibilityListening = listen;
  if (listen) {
    document.addEventListener("visibilitychange", handleVisibilityChange);
  } else {
    document.removeEventListener("visibilitychange", handleVisibilityChange);
  }
}

function clearPollTimer(): void {
  if (pollTimer !== null) {
    clearTimeout(pollTimer);
    pollTimer = null;
  }
}

/** Arms the single poll timer unless one is armed, a round is running, or the tab is hidden. */
function schedulePoll(delayMs: number): void {
  if (pollTimer !== null || inFlight || documentHidden() || !hasActiveBatches()) {
    return;
  }
  pollTimer = setTimeout(() => {
    pollTimer = null;
    void pollAll();
  }, delayMs);
}

function ensurePolling(): void {
  setVisibilityListening(true);
  schedulePoll(nextPollDelay(consecutiveFailedRounds));
}

function maybeStopPolling(): void {
  if (hasActiveBatches()) return;
  clearPollTimer();
  setVisibilityListening(false);
}

async function pollAll(): Promise<void> {
  if (inFlight) return;

  const ids = Object.keys(batches).filter((id) => {
    const entry = batches[id];
    return entry !== undefined && isActive(entry);
  });
  if (ids.length === 0) {
    maybeStopPolling();
    return;
  }

  inFlight = true;
  let roundFailed = false;
  try {
    const results = await Promise.allSettled(
      ids.map((id) => photographyApi.getBatchStatus(id)),
    );

    let completedNow = false;
    results.forEach((result, index) => {
      const id = ids[index];
      if (id === undefined) return;
      const existing = batches[id];
      if (!existing) return;

      if (result.status === "fulfilled") {
        const status = result.value.data;
        setBatches((state) => {
          const entry = state[id];
          if (!entry) return;
          entry.status = status;
          entry._missing = false;
          if (status.done && !entry._gridRefreshed) {
            entry._gridRefreshed = true;
            completedNow = true;
          }
        });
      } else {
        const reason = result.reason as { status?: number } | undefined;
        if (reason?.status === 404) {
          if (Date.now() - existing._firstSeen > MISSING_GRACE_MS) {
            setBatches((state) => {
              const entry = state[id];
              if (entry) entry._missing = true;
            });
          }
        } else {
          // Server or network error: leave state untouched and back off.
          roundFailed = true;
        }
      }
    });

    if (completedNow && completionHandler) {
      completionHandler();
    }
  } finally {
    inFlight = false;
    consecutiveFailedRounds = roundFailed ? consecutiveFailedRounds + 1 : 0;
    flush();
    if (hasActiveBatches()) {
      schedulePoll(nextPollDelay(consecutiveFailedRounds));
    } else {
      maybeStopPolling();
    }
  }
}

/** Test hook: stop polling and forget all tracked batches and backoff state. */
export function resetBatchPollingForTests(): void {
  clearPollTimer();
  setVisibilityListening(false);
  inFlight = false;
  consecutiveFailedRounds = 0;
  setBatches((state) => {
    for (const id of Object.keys(state)) delete state[id];
  });
  flush();
}
