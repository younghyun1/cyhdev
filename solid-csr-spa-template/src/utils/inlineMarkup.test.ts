import { describe, expect, it } from "vitest";
import { inlineToPlainText, isSafeHref, parseInline } from "./inlineMarkup";

describe("parseInline", () => {
  it("splits strong, code, and link segments", () => {
    expect(
      parseInline("Built **Sillok** with `Turso` and [docs](https://example.com/a)."),
    ).toEqual([
      { kind: "text", text: "Built " },
      { kind: "strong", text: "Sillok" },
      { kind: "text", text: " with " },
      { kind: "code", text: "Turso" },
      { kind: "text", text: " and " },
      {
        kind: "link",
        href: "https://example.com/a",
        external: true,
        label: [{ kind: "text", text: "docs" }],
      },
      { kind: "text", text: "." },
    ]);
  });

  it("treats same-origin paths as internal links and keeps code in labels", () => {
    expect(parseInline("[the `main` page](/about)")).toEqual([
      {
        kind: "link",
        href: "/about",
        external: false,
        label: [
          { kind: "text", text: "the " },
          { kind: "code", text: "main" },
          { kind: "text", text: " page" },
        ],
      },
    ]);
  });

  it("renders unsafe link targets as plain label text", () => {
    expect(parseInline("[click](javascript:alert(1))")).toEqual([
      { kind: "text", text: "click" },
      { kind: "text", text: ")" },
    ]);
    expect(parseInline("[proto](//evil.example)")).toEqual([
      { kind: "text", text: "proto" },
    ]);
  });

  it("leaves unmatched markers literal", () => {
    expect(parseInline("a ** b ` c [d")).toEqual([
      { kind: "text", text: "a ** b ` c [d" },
    ]);
    expect(parseInline("$5,000 -> $150")).toEqual([
      { kind: "text", text: "$5,000 -> $150" },
    ]);
  });

  it("bounds oversized input", () => {
    const parsed = parseInline("x".repeat(10_000));
    expect(parsed).toHaveLength(1);
    expect(parsed[0]?.kind === "text" && parsed[0].text.length).toBe(8192);
  });
});

describe("isSafeHref", () => {
  it.each([
    ["/about/experience", true],
    ["https://github.com/younghyun1", true],
    ["mailto:younghyun1@gmail.com", true],
    ["http://insecure.example", false],
    ["//protocol-relative.example", false],
    ["data:text/html,hi", false],
  ])("%s -> %s", (href, expected) => {
    expect(isSafeHref(href)).toBe(expected);
  });
});

describe("inlineToPlainText", () => {
  it("drops markers and keeps link labels", () => {
    expect(inlineToPlainText("**Rust** at [cyhdev](/) with `Axum`")).toBe(
      "Rust at cyhdev with Axum",
    );
  });
});
