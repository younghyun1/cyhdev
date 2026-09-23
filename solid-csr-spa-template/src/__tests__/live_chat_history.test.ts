import { describe, expect, it } from "vitest";
import {
  appendLive,
  bufferDetached,
  mergeLatest,
  prependOlder,
  trimOldest,
} from "../services/live_chat_history";

const LIMIT = 5;
const range = (from: number, to: number) =>
  Array.from({ length: to - from + 1 }, (_, index) => from + index);

describe("live chat history window", () => {
  it("trims the oldest entries at the live edge", () => {
    const update = appendLive(range(1, 5), 6, true, LIMIT);
    expect(update.list).toEqual(range(2, 6));
    expect(update).toMatchObject({ detached: false, trimmedOldest: true });
  });

  it("appends while there is room even when the reader is in history", () => {
    const update = appendLive(range(1, 3), 4, false, LIMIT);
    expect(update.list).toEqual(range(1, 4));
    expect(update.detached).toBe(false);
  });

  it("never trims under a reader scrolled into a full window", () => {
    const loaded = range(1, 5);
    const update = appendLive(loaded, 6, false, LIMIT);
    expect(update.list).toEqual(loaded);
    expect(update).toMatchObject({ detached: true, trimmedOldest: false });
  });

  it("keeps the older page just loaded and drops the newest instead", () => {
    const update = prependOlder(range(4, 7), range(1, 3), LIMIT);
    expect(update.list).toEqual(range(1, 5));
    expect(update.detached).toBe(true);
    const fits = prependOlder(range(3, 4), range(1, 2), LIMIT);
    expect(fits).toMatchObject({ list: range(1, 4), detached: false });
  });

  it("bounds live growth after an acknowledgement", () => {
    expect(trimOldest(range(1, 7), LIMIT).list).toEqual(range(3, 7));
    expect(trimOldest(range(1, 3), LIMIT).trimmedOldest).toBe(false);
  });

  it("buffers detached messages boundedly and merges them with the latest page", () => {
    let buffer: number[] = [];
    for (const value of range(1, 8)) buffer = bufferDetached(buffer, value, 3);
    expect(buffer).toEqual([6, 7, 8]);

    const message = (id: string, second: number) => ({
      id,
      createdAt: new Date(Date.UTC(2026, 8, 23, 12, 0, second)).toISOString(),
    });
    const latest = [message("a", 1), message("c", 3), message("b", 3)];
    const buffered = [message("c", 3), message("d", 4)];
    const merged = mergeLatest(latest, buffered, (item) => item, LIMIT);
    expect(merged.map((item) => item.id)).toEqual(["a", "b", "c", "d"]);
    expect(mergeLatest(latest, buffered, (item) => item, 2).map((item) => item.id)).toEqual([
      "c",
      "d",
    ]);
  });
});
