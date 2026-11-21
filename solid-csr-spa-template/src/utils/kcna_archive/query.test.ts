import { describe, expect, it } from "vitest";

import {
  EMPTY_QUERY,
  firstParam,
  isFiltered,
  parseQuery,
  queryFromString,
  queryParams,
  queryString,
  toggled,
  withoutRange,
} from "./query";

describe("parseQuery", () => {
  it("reads every filter from URL parameters", () => {
    expect(
      parseQuery({
        q: " 김정은 ",
        lang: "eng,kor",
        source: "kcna_kp",
        category: "국내소식",
        from: "2026-09-01",
        to: "2026-11-21",
        sort: "oldest",
      }),
    ).toEqual({
      text: "김정은",
      languages: ["kor", "eng"],
      sources: ["kcna_kp"],
      category: "국내소식",
      from: "2026-09-01",
      to: "2026-11-21",
      sort: "oldest",
    });
  });

  it("ignores invalid values instead of failing", () => {
    expect(
      parseQuery({ lang: "kor,xx", source: "elsewhere", from: "2026-02-30", to: "soon", sort: "random" }),
    ).toEqual({ ...EMPTY_QUERY, languages: ["kor"] });
    expect(parseQuery({})).toEqual(EMPTY_QUERY);
  });

  it("drops a reversed date range and caps the text length", () => {
    const query = parseQuery({ from: "2026-11-02", to: "2026-11-01", q: "a".repeat(300) });
    expect([query.from, query.to]).toEqual([null, null]);
    expect(query.text).toHaveLength(200);
  });

  it("takes the first of a repeated parameter", () => {
    expect(firstParam(["a", "b"])).toBe("a");
    expect(firstParam([])).toBeUndefined();
    expect(parseQuery({ q: ["first", "second"] }).text).toBe("first");
  });
});

describe("query serialization", () => {
  it("round-trips through URL parameters and through its string form", () => {
    const query = parseQuery({ q: "핵무력 강화", lang: "kor", from: "2018-01-01", sort: "newest" });
    const params = queryParams(query);
    expect(parseQuery(params)).toEqual(query);
    expect(params.source).toBeUndefined();
    expect(params.to).toBeUndefined();
    expect(queryFromString(queryString(query))).toEqual(query);
  });

  it("omits defaults and encodes text", () => {
    expect(queryString(EMPTY_QUERY)).toBe("");
    expect(queryString(EMPTY_QUERY, { limit: 50, offset: undefined })).toBe("limit=50");
    const text = queryString({ ...EMPTY_QUERY, text: "a&b c" });
    expect(new URLSearchParams(text).get("q")).toBe("a&b c");
  });

  it("sorts newest first by default and names any other order", () => {
    expect(parseQuery({ q: "평화" }).sort).toBe("newest");
    expect(queryParams(parseQuery({ q: "평화" })).sort).toBeUndefined();
    expect(queryParams(parseQuery({ q: "평화", sort: "relevance" })).sort).toBe("relevance");
  });

  it("gives equal selections equal keys regardless of order", () => {
    expect(queryString(parseQuery({ lang: "eng,kor" }))).toBe(queryString(parseQuery({ lang: "kor,eng" })));
  });
});

describe("query helpers", () => {
  it("removes the date range and sort for the histogram", () => {
    const query = parseQuery({ q: "평화", from: "2020-01-01", to: "2020-12-31", sort: "oldest" });
    expect(withoutRange(query)).toEqual({ ...EMPTY_QUERY, text: "평화" });
  });

  it("toggles list members and detects any filter", () => {
    expect(toggled(["kor"], "eng")).toEqual(["kor", "eng"]);
    expect(toggled(["kor", "eng"], "kor")).toEqual(["eng"]);
    expect(isFiltered(EMPTY_QUERY)).toBe(false);
    expect(isFiltered({ ...EMPTY_QUERY, languages: ["kor"] })).toBe(true);
  });
});
