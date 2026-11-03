import { describe, expect, it } from "vitest";
import { fuzzyScore, rankByQuery } from "./fuzzyMatch";

describe("fuzzyScore", () => {
  it("prefers prefixes and word starts over inner substrings", () => {
    const prefix = fuzzyScore("ab", "About");
    const wordStart = fuzzyScore("blog", "About › Blog");
    const inner = fuzzyScore("bout", "About");
    expect(prefix).toBe(0);
    expect(wordStart).not.toBeNull();
    expect(inner).not.toBeNull();
    expect(wordStart!).toBeLessThan(inner!);
  });

  it("falls back to ordered subsequences with a gap penalty", () => {
    expect(fuzzyScore("pfm", "Performance")).toBe(100 + 4);
    expect(fuzzyScore("mfp", "Performance")).toBeNull();
  });

  it("ignores case and diacritics", () => {
    expect(fuzzyScore("uber", "Über mich")).toBe(0);
    expect(fuzzyScore("ESPAÑ", "español")).toBe(0);
  });

  it("matches scripts without letter case", () => {
    expect(fuzzyScore("경력", "소개 › 경력")).not.toBeNull();
    expect(fuzzyScore("技术", "博客技术栈")).not.toBeNull();
  });

  it("treats an empty query as a match", () => {
    expect(fuzzyScore("   ", "Anything")).toBe(0);
  });
});

describe("rankByQuery", () => {
  const entries = [
    { label: "Photographs", href: "/photographs" },
    { label: "Projects", href: "/projects" },
    { label: "About › Projects", href: "/about/projects" },
    { label: "Live chat", href: "/live-chat" },
  ];

  it("filters and orders by best field, keeping input order on ties", () => {
    const ranked = rankByQuery(entries, "proj", (entry) => [entry.label, entry.href]);
    expect(ranked.map((entry) => entry.href)).toEqual(["/projects", "/about/projects"]);
  });

  it("returns every entry in order for an empty query", () => {
    expect(rankByQuery(entries, "", (entry) => [entry.label])).toEqual(entries);
  });
});
