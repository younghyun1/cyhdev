import { afterEach, describe, expect, it, vi } from "vitest";

import { exportHref, kcnaArchiveApi } from "../services/contracts/kcna_archive";
import { EMPTY_QUERY } from "../utils/kcna_archive/query";

const envelope = (data: unknown) => ({ success: true, data, meta: {} });

const stubFetch = (data: unknown, status = 200) => {
  const fetchMock = vi.fn(
    async (_input: RequestInfo | URL, _init?: RequestInit) =>
      new Response(JSON.stringify(status === 200 ? envelope(data) : data), { status }),
  );
  vi.stubGlobal("fetch", fetchMock);
  return fetchMock;
};

const requested = (fetchMock: ReturnType<typeof stubFetch>): URL => {
  const input = fetchMock.mock.calls[0]?.[0];
  return new URL(String(input), "http://localhost");
};

afterEach(() => vi.unstubAllGlobals());

describe("archive service wrapper", () => {
  it("omits defaults from a search and always sends the page size", async () => {
    const fetchMock = stubFetch({ total: 0 });
    await kcnaArchiveApi.search(EMPTY_QUERY, 0);
    const url = requested(fetchMock);
    expect(url.pathname).toBe("/api/admin/kcna-archive/search");
    expect(url.search).toBe("?limit=50");
  });

  it("sends every filter, the offset, and the page size", async () => {
    const fetchMock = stubFetch({ total: 0 });
    await kcnaArchiveApi.search(
      {
        text: "핵 무력",
        languages: ["kor", "eng"],
        sources: ["kcna_kp"],
        category: "Economy",
        from: "2026-01-01",
        to: "2026-02-28",
        sort: "relevance",
      },
      100,
    );
    const params = requested(fetchMock).searchParams;
    expect(Object.fromEntries(params)).toEqual({
      q: "핵 무력",
      lang: "kor,eng",
      source: "kcna_kp",
      category: "Economy",
      from: "2026-01-01",
      to: "2026-02-28",
      sort: "relevance",
      offset: "100",
      limit: "50",
    });
  });

  it("asks the histogram for its bucket size", async () => {
    const fetchMock = stubFetch({ bucket: "month", buckets: [] });
    await kcnaArchiveApi.histogram({ ...EMPTY_QUERY, text: "peace" }, "month");
    const url = requested(fetchMock);
    expect(url.pathname).toBe("/api/admin/kcna-archive/histogram");
    expect(url.search).toBe("?q=peace&bucket=month");
  });

  it("encodes the document id and passes the highlight text only when there is one", async () => {
    const plain = stubFetch({ id: "x" });
    await kcnaArchiveApi.document("a/b", "");
    expect(requested(plain).pathname).toBe("/api/admin/kcna-archive/documents/a%2Fb");
    expect(requested(plain).search).toBe("");

    const marked = stubFetch({ id: "x" });
    await kcnaArchiveApi.document("x", "김 정");
    expect(requested(marked).search).toBe("?q=%EA%B9%80+%EC%A0%95");
  });

  it("repeats the term parameter and drops the search text for trends", async () => {
    const fetchMock = stubFetch({ bucket: "year", starts: [], totals: [], series: [] });
    await kcnaArchiveApi.trends(
      ["핵무력", "경제"],
      { ...EMPTY_QUERY, text: "ignored", languages: ["kor"] },
      "year",
    );
    const params = requested(fetchMock).searchParams;
    expect(params.getAll("term")).toEqual(["핵무력", "경제"]);
    expect(params.has("q")).toBe(false);
    expect(params.get("lang")).toBe("kor");
    expect(params.get("bucket")).toBe("year");
  });

  it("reloads with a POST and unwraps the status", async () => {
    const fetchMock = stubFetch({ state: "ready", reloading: false, lastReloadFailed: false });
    await expect(kcnaArchiveApi.reload()).resolves.toMatchObject({ state: "ready" });
    expect(requested(fetchMock).pathname).toBe("/api/admin/kcna-archive/reload");
    expect(fetchMock.mock.calls[0]?.[1]?.method).toBe("POST");
  });

  it("sends credentials with every request", async () => {
    const fetchMock = stubFetch({ state: "ready" });
    await kcnaArchiveApi.status();
    expect(fetchMock.mock.calls[0]?.[1]?.credentials).toBe("include");
  });

  it("rejects with the API error so pages can tell a missing snapshot from a fault", async () => {
    stubFetch({ success: false, error_code: 113 }, 503);
    await expect(kcnaArchiveApi.status()).rejects.toMatchObject({ status: 503 });
  });
});

describe("export links", () => {
  it("point at the export endpoint with the current query and the format", () => {
    const href = exportHref({ ...EMPTY_QUERY, text: "평화", languages: ["kor"] }, "jsonl");
    const url = new URL(href, "http://localhost");
    expect(url.pathname).toBe("/api/admin/kcna-archive/export");
    expect(Object.fromEntries(url.searchParams)).toEqual({ q: "평화", lang: "kor", format: "jsonl" });
    expect(exportHref(EMPTY_QUERY, "csv")).toBe("/api/admin/kcna-archive/export?format=csv");
  });
});
