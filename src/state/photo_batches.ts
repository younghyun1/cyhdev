// Module-scoped global store + poller for photograph batch-upload sessions.
//
// Survives route navigation (module singleton, like state/auth.ts). A single
// shared interval polls all active batches with an in-flight guard; it stops
// when nothing is active and restarts on trackBatch. A 404 from the poller marks
// a batch _missing (after a short grace) rather than triggering logout, because
// the status endpoint returns 404 (never 403) for absent/not-owned batches.

import { createStore, produce } from "solid-js/store";
import { photographyApi } from "../services/all_api";
import type {
  BatchStatusResponse,
  BatchUploadResponse,
} from "../dtos/responses/photography";

export interface BatchEntry {
  batch_id: string;
  status: BatchStatusResponse | null;
  _missing: boolean;
  _firstSeen: number;
  _gridRefreshed: boolean;
}

const [batches, setBatches] = createStore<Record<string, BatchEntry>>({});
export { batches };

const POLL_INTERVAL_MS = 1200;
const MISSING_GRACE_MS = 6000;

let intervalId: ReturnType<typeof setInterval> | null = null;
let inFlight = false;
let completionHandler: (() => void) | null = null;

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
  setBatches(initial.batch_id, {
    batch_id: initial.batch_id,
    status: initial,
    _missing: false,
    _firstSeen: Date.now(),
    _gridRefreshed: false,
  });
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
  setBatches(
    produce((state) => {
      for (const id of Object.keys(state)) {
        const entry = state[id];
        if (entry && (entry._missing || entry.status?.done)) {
          delete state[id];
        }
      }
    }),
  );
  maybeStopPolling();
}

function ensurePolling(): void {
  if (intervalId === null) {
    intervalId = setInterval(() => {
      void pollAll();
    }, POLL_INTERVAL_MS);
  }
}

function maybeStopPolling(): void {
  let anyActive = false;
  for (const id of Object.keys(batches)) {
    const entry = batches[id];
    if (entry && isActive(entry)) {
      anyActive = true;
      break;
    }
  }
  if (!anyActive && intervalId !== null) {
    clearInterval(intervalId);
    intervalId = null;
  }
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
        setBatches(id, "status", status);
        setBatches(id, "_missing", false);
        if (status.done && !existing._gridRefreshed) {
          setBatches(id, "_gridRefreshed", true);
          completedNow = true;
        }
      } else {
        const reason = result.reason as { status?: number } | undefined;
        if (
          reason?.status === 404 &&
          Date.now() - existing._firstSeen > MISSING_GRACE_MS
        ) {
          setBatches(id, "_missing", true);
        }
        // Other errors: leave state untouched and retry next tick.
      }
    });

    if (completedNow && completionHandler) {
      completionHandler();
    }
  } finally {
    inFlight = false;
    maybeStopPolling();
  }
}
