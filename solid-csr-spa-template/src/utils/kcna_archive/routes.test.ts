import { describe, expect, it } from "vitest";

import { EMPTY_QUERY } from "./query";
import {
  ARCHIVE_PATH,
  besideLanguageOf,
  documentHref,
  searchHref,
  sectionOf,
} from "./routes";

describe("archive links", () => {
  it("carries the search into the reader and back", () => {
    const query = { ...EMPTY_QUERY, text: "bridge", languages: ["eng" as const] };
    expect(documentHref("abc", query)).toBe(`${ARCHIVE_PATH}/documents/abc?q=bridge&lang=eng`);
    expect(documentHref("abc", EMPTY_QUERY)).toBe(`${ARCHIVE_PATH}/documents/abc`);
    expect(searchHref(query, { doc: "abc" })).toBe(`${ARCHIVE_PATH}?q=bridge&lang=eng&doc=abc`);
    expect(searchHref(EMPTY_QUERY)).toBe(ARCHIVE_PATH);
  });

  it("encodes the document id", () => {
    expect(documentHref("a/b", EMPTY_QUERY)).toBe(`${ARCHIVE_PATH}/documents/a%2Fb`);
  });

  it("accepts only an archive language for the side-by-side view", () => {
    expect(besideLanguageOf("eng")).toBe("eng");
    expect(besideLanguageOf(["zho", "kor"])).toBe("zho");
    expect(besideLanguageOf("fra")).toBeUndefined();
    expect(besideLanguageOf(undefined)).toBeUndefined();
  });

  it("assigns pathnames to tabs, counting the reader as search", () => {
    expect(sectionOf(ARCHIVE_PATH)).toBe("search");
    expect(sectionOf(`${ARCHIVE_PATH}/`)).toBe("search");
    expect(sectionOf(`${ARCHIVE_PATH}/documents/abc`)).toBe("search");
    expect(sectionOf(`${ARCHIVE_PATH}/trends`)).toBe("trends");
    expect(sectionOf(`${ARCHIVE_PATH}/status`)).toBe("status");
    expect(sectionOf("/projects")).toBeUndefined();
  });
});
