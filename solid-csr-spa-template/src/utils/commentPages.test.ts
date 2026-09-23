import { describe, expect, it } from "vitest";
import { mergeCommentPages } from "./commentPages";

type Row = { readonly id: string; readonly text: string };

const idOf = (row: Row) => row.id;
const none = () => undefined;

describe("comment page accumulation", () => {
  it("keeps page order and drops rows repeated across pages", () => {
    const first = [
      { id: "a", text: "parent" },
      { id: "b", text: "reply" },
    ];
    const second = [
      { id: "b", text: "reply again" },
      { id: "c", text: "late reply" },
    ];
    expect(mergeCommentPages([first, second], idOf, none)).toEqual([
      { id: "a", text: "parent" },
      { id: "b", text: "reply" },
      { id: "c", text: "late reply" },
    ]);
  });

  it("substitutes local edits and tombstones for fetched copies", () => {
    const tombstone = { id: "a", text: "" };
    const merged = mergeCommentPages(
      [[{ id: "a", text: "parent" }], [{ id: "b", text: "reply" }]],
      idOf,
      (id) => (id === "a" ? tombstone : undefined),
    );
    expect(merged).toEqual([tombstone, { id: "b", text: "reply" }]);
  });
});
