import { createRoot, createSignal, flush } from "solid-js";
import { afterEach, describe, expect, it, vi } from "vitest";
import {
  chartTimeFormatter,
  createTimeLabels,
} from "../components/chartTimeLabels";
import { setLocaleSignal } from "../state/i18n";
import { webSocketUrl } from "../utils/webSocketUrl";

afterEach(() => {
  setLocaleSignal("en-US");
  flush();
  vi.restoreAllMocks();
});

describe("chart time labels", () => {
  it("reuses one formatter per locale", () => {
    expect(chartTimeFormatter("en-US")).toBe(chartTimeFormatter("en-US"));
    expect(chartTimeFormatter("ko-KR")).not.toBe(chartTimeFormatter("en-US"));
  });

  it("formats only new samples as the window slides and pads to the limit", () => {
    const base = Date.UTC(2026, 8, 23, 12, 0, 0);
    const expected = chartTimeFormatter("en-US").format(base + 1_000);
    // `format` is a prototype getter returning a bound function; each call
    // reads it once. The DOM typings model it as a method, hence the cast.
    const format = vi.spyOn(
      Intl.DateTimeFormat.prototype as unknown as { readonly format: unknown },
      "format",
      "get",
    );
    const [samples, setSamples] = createSignal([{ ts: base }, { ts: base + 1_000 }]);
    const { labels, dispose } = createRoot((dispose) => ({
      labels: createTimeLabels(samples, () => 4),
      dispose,
    }));
    expect(labels()).toHaveLength(4);
    expect(labels().slice(2)).toEqual(["", ""]);
    expect(format).toHaveBeenCalledTimes(2);

    setSamples([{ ts: base + 1_000 }, { ts: base + 2_000 }]);
    flush();
    expect(labels()[0]).toBe(expected);
    expect(format).toHaveBeenCalledTimes(3);
    dispose();
  });

  it("reformats every label when the locale changes", () => {
    const ts = Date.UTC(2026, 8, 23, 12, 0, 0);
    const { labels, dispose } = createRoot((dispose) => ({
      labels: createTimeLabels(() => [{ ts }], () => 1),
      dispose,
    }));
    expect(labels()[0]).toBe(chartTimeFormatter("en-US").format(ts));
    setLocaleSignal("ko-KR");
    flush();
    expect(labels()[0]).toBe(chartTimeFormatter("ko-KR").format(ts));
    dispose();
  });
});

describe("WebSocket URLs", () => {
  it("builds an absolute same-origin socket URL when no API base is set", () => {
    expect(webSocketUrl("/ws/host-stats", "", "https://cyhdev.example")).toBe(
      "wss://cyhdev.example/ws/host-stats",
    );
    expect(webSocketUrl("/ws/host-stats", "", "http://127.0.0.1:3000")).toBe(
      "ws://127.0.0.1:3000/ws/host-stats",
    );
  });

  it("follows a configured API base and keeps its path prefix", () => {
    expect(
      webSocketUrl("/ws/host-stats", "https://api.example/backend/", "http://127.0.0.1:3000"),
    ).toBe("wss://api.example/backend/ws/host-stats");
    expect(webSocketUrl("/ws/host-stats", "/backend", "https://cyhdev.example")).toBe(
      "wss://cyhdev.example/backend/ws/host-stats",
    );
  });
});
