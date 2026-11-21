import { describe, expect, it } from "vitest";

import { RESULT_WINDOW } from "./constants";
import {
  createRequestGate,
  nextOffset,
  noLaterPages,
  pageFailed,
  pageReceived,
  pageRequested,
  pagesFor,
} from "./sequence";

const item = (id: string) => ({ id });

describe("request gate", () => {
  it("lets only the newest ticket apply its result", () => {
    const gate = createRequestGate();
    const first = gate.begin();
    expect(gate.isCurrent(first)).toBe(true);
    const second = gate.begin();
    expect(gate.isCurrent(first)).toBe(false);
    expect(gate.isCurrent(second)).toBe(true);
  });

  it("makes every outstanding ticket stale when invalidated", () => {
    const gate = createRequestGate();
    const ticket = gate.begin();
    gate.invalidate();
    expect(gate.isCurrent(ticket)).toBe(false);
  });
});

describe("later result pages", () => {
  it("starts empty for a query it has not seen", () => {
    const state = pageReceived(noLaterPages<{ id: string }>(), "q=a", [item("1")]);
    expect(pagesFor(state, "q=a").items).toEqual([item("1")]);
    expect(pagesFor(state, "q=b").items).toEqual([]);
  });

  it("appends pages without repeating an article", () => {
    let state = pageReceived(noLaterPages<{ id: string }>(), "k", [item("1"), item("2")]);
    state = pageReceived(state, "k", [item("2"), item("3")]);
    expect(state.items.map((entry) => entry.id)).toEqual(["1", "2", "3"]);
    expect(state.loading).toBe(false);
  });

  it("tracks loading and failure, and clears the failure on the next request", () => {
    let state = pageRequested(noLaterPages<{ id: string }>(), "k");
    expect(state.loading).toBe(true);
    state = pageFailed(state, "k", "network");
    expect(state).toMatchObject({ loading: false, failure: "network" });
    state = pageRequested(state, "k");
    expect(state).toMatchObject({ loading: true, failure: undefined });
  });

  it("drops what belongs to another query when a new one is requested", () => {
    const state = pageReceived(noLaterPages<{ id: string }>(), "old", [item("1")]);
    const next = pageRequested(state, "new");
    expect(next.key).toBe("new");
    expect(next.items).toEqual([]);
  });

  it("stops at the server's result window", () => {
    expect(nextOffset(noLaterPages())).toBe(50);
    const full = { ...noLaterPages<{ id: string }>(), items: Array.from({ length: RESULT_WINDOW - 50 }, (_, index) => item(String(index))) };
    expect(nextOffset(full)).toBeUndefined();
    const nearly = pageReceived({ ...noLaterPages<{ id: string }>(), key: "k", items: full.items.slice(0, RESULT_WINDOW - 60) }, "k", Array.from({ length: 30 }, (_, index) => item(`new-${index}`)));
    expect(nearly.items.length).toBe(RESULT_WINDOW - 50);
  });
});
