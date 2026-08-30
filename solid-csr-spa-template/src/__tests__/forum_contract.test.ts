import { beforeEach, describe, expect, it, vi } from "vitest";

import { forumApi } from "../services/contracts/forum";

const responseBody = (data: unknown) =>
  new Response(
    JSON.stringify({
      success: true,
      data,
      meta: {
        time_to_process: "redacted",
        timestamp: "2026-08-30T12:00:00Z",
        metadata: null,
      },
    }),
    { status: 200, headers: { "content-type": "application/json" } },
  );

describe("forum HTTP contract", () => {
  const fetchMock = vi.fn();

  beforeEach(() => {
    fetchMock.mockReset();
    vi.stubGlobal("fetch", fetchMock);
  });

  it("encodes the complete topic keyset cursor", async () => {
    fetchMock.mockResolvedValue(
      responseBody({ topics: [], next_cursor: null }),
    );

    await forumApi.topics(
      "latency budget",
      {
        before_pinned: false,
        before_activity_at: "2026-08-30T12:00:00Z",
        before_topic_id: "0198f4d0-aaaa-7000-8000-000000000001",
      },
      25,
    );

    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    const parsed = new URL(url, "https://example.test");
    expect(parsed.pathname).toBe("/api/forum/topics");
    expect(parsed.searchParams.get("search")).toBe("latency budget");
    expect(parsed.searchParams.get("before_pinned")).toBe("false");
    expect(parsed.searchParams.get("before_activity_at")).toBe(
      "2026-08-30T12:00:00Z",
    );
    expect(parsed.searchParams.get("before_topic_id")).toBe(
      "0198f4d0-aaaa-7000-8000-000000000001",
    );
    expect(init.credentials).toBe("include");
  });

  it("sends moderation revision and reason in JSON", async () => {
    fetchMock.mockResolvedValue(
      responseBody({
        audit_event_id: "0198f4d0-bbbb-7000-8000-000000000001",
        target_id: "0198f4d0-aaaa-7000-8000-000000000001",
        revision: 4,
        action: "topic_locked",
        moderated_at: "2026-08-30T12:00:00Z",
      }),
    );

    await forumApi.moderateTopic(
      "0198f4d0-aaaa-7000-8000-000000000001",
      { action: "lock", reason: "Thread needs review", expected_revision: 3 },
    );

    const [url, init] = fetchMock.mock.calls[0] as [string, RequestInit];
    expect(url).toBe(
      "/api/forum/topics/0198f4d0-aaaa-7000-8000-000000000001/moderation",
    );
    expect(init.method).toBe("POST");
    expect(JSON.parse(String(init.body))).toEqual({
      action: "lock",
      reason: "Thread needs review",
      expected_revision: 3,
    });
  });
});
