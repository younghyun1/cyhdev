import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const fetchHealthState = vi.fn();
vi.mock("../services/api", () => ({ fetchHealthState }));

const {
  HEALTH_REFRESH_TTL_MS,
  refreshHealthState,
  refreshHealthStateIfStale,
  resetHealthRefreshTiming,
} = await import("../state/health");

const response = {
  success: true,
  data: {
    db_latency: "1ms",
    db_version: "PostgreSQL test",
    responses_handled: 1,
    server_uptime: "1 hour",
    timestamp: "2026-09-23T00:00:00Z",
    users_logged_in: 1,
  },
  meta: { time_to_process: "1ms" },
};

beforeEach(() => {
  resetHealthRefreshTiming();
  fetchHealthState.mockReset();
  fetchHealthState.mockResolvedValue(response);
});

afterEach(() => {
  resetHealthRefreshTiming();
  vi.restoreAllMocks();
});

describe("health refresh throttling", () => {
  it("shares one in-flight request across bursts of callers", async () => {
    let resolve: (value: typeof response) => void = () => {};
    fetchHealthState.mockReturnValue(
      new Promise<typeof response>((next) => (resolve = next)),
    );
    const first = refreshHealthStateIfStale();
    const second = refreshHealthStateIfStale();
    expect(second).toBe(first);
    resolve(response);
    await first;
    expect(fetchHealthState).toHaveBeenCalledTimes(1);
  });

  it("skips refreshes inside the TTL and refreshes once it has elapsed", async () => {
    await refreshHealthStateIfStale();
    const started = performance.now();
    await refreshHealthStateIfStale(started + HEALTH_REFRESH_TTL_MS - 1_000);
    expect(fetchHealthState).toHaveBeenCalledTimes(1);
    await refreshHealthStateIfStale(started + HEALTH_REFRESH_TTL_MS + 1_000);
    expect(fetchHealthState).toHaveBeenCalledTimes(2);
  });

  it("keeps the TTL after a failed refresh so outages are not hammered", async () => {
    vi.spyOn(console, "error").mockImplementation(() => {});
    fetchHealthState.mockRejectedValueOnce(new Error("offline"));
    await refreshHealthStateIfStale();
    await refreshHealthStateIfStale();
    expect(fetchHealthState).toHaveBeenCalledTimes(1);
  });

  it("lets explicit refreshes bypass the TTL", async () => {
    await refreshHealthStateIfStale();
    await refreshHealthState();
    expect(fetchHealthState).toHaveBeenCalledTimes(2);
  });
});
