import { describe, expect, it } from "vitest";
import { parseForumBody, quoteForReply } from "./forumBody";

describe("parseForumBody", () => {
  it("separates quote blocks from text", () => {
    const blocks = parseForumBody("> a wrote:\n> first\n\nanswer\nsecond line");
    expect(blocks.map((block) => block.kind)).toEqual(["quote", "text"]);
    expect(blocks[0]?.inline).toEqual([{ kind: "text", text: "a wrote:\nfirst" }]);
    expect(blocks[1]?.inline).toEqual([{ kind: "text", text: "answer\nsecond line" }]);
  });

  it("links only http and https URLs and leaves trailing punctuation out", () => {
    const [block] = parseForumBody("see https://example.com/a?b=1. or javascript:alert(1)");
    expect(block?.inline).toEqual([
      { kind: "text", text: "see " },
      { kind: "link", text: "https://example.com/a?b=1", href: "https://example.com/a?b=1" },
      { kind: "text", text: ". or javascript:alert(1)" },
    ]);
  });

  it("keeps markup as text", () => {
    const [block] = parseForumBody('<img src=x onerror="bad()">');
    expect(block?.inline).toEqual([{ kind: "text", text: '<img src=x onerror="bad()">' }]);
  });

  it("returns nothing for blank bodies", () => {
    expect(parseForumBody("\n\n")).toEqual([]);
  });

  it("drops blank lines around a block but keeps them inside it", () => {
    const [block] = parseForumBody(`\n\na${"\n".repeat(19_000)}b\n\n`);
    expect(block?.inline).toEqual([{ kind: "text", text: `a${"\n".repeat(19_000)}b` }]);
  });

  it("keeps punctuation that is not at the end of a URL", () => {
    const url = `http://a.test/${"!".repeat(19_000)}x`;
    const [block] = parseForumBody(`${url}!!!`);
    expect(block?.inline).toEqual([
      { kind: "link", text: url, href: url },
      { kind: "text", text: "!!!" },
    ]);
  });
});

describe("quoteForReply", () => {
  it("prefixes every line and drops earlier quotes", () => {
    expect(quoteForReply("a wrote:", "> old\nnew\n\nmore")).toBe(
      "> a wrote:\n> new\n>\n> more\n\n",
    );
  });

  it("caps very long quotes", () => {
    const quoted = quoteForReply("a wrote:", "x".repeat(5_000));
    expect(Array.from(quoted).length).toBeLessThan(2_100);
    expect(quoted).toContain("…");
  });
});
