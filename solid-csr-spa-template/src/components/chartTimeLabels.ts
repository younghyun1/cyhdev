import { type Accessor, createMemo } from "solid-js";
import type { UiLocale } from "../i18n/keys";
import { locale } from "../state/i18n";

/** One formatter per UI locale; the key set is the fixed locale list, so it cannot grow. */
const formatters = new Map<UiLocale, Intl.DateTimeFormat>();

/** Shared 24-hour HH:MM:SS formatter for streamed chart axes. */
export function chartTimeFormatter(tag: UiLocale): Intl.DateTimeFormat {
  let formatter = formatters.get(tag);
  if (!formatter) {
    formatter = new Intl.DateTimeFormat(tag, {
      hour: "2-digit",
      minute: "2-digit",
      second: "2-digit",
      hourCycle: "h23",
    });
    formatters.set(tag, formatter);
  }
  return formatter;
}

/**
 * Axis labels for a sliding window of timestamped samples, padded with blanks
 * to the window size `limit`. The window advances one sample per tick, so
 * labels are reused by timestamp and only new samples are formatted. The reuse
 * map is rebuilt from the current window on every run, which bounds it to the
 * window size, and it is discarded when the UI locale changes.
 */
export function createTimeLabels(
  samples: Accessor<readonly { readonly ts: number }[]>,
  limit: Accessor<number>,
): Accessor<string[]> {
  let cachedLocale: UiLocale | null = null;
  let cached = new Map<number, string>();
  return createMemo(() => {
    const tag = locale();
    const formatter = chartTimeFormatter(tag);
    const previous = cachedLocale === tag ? cached : new Map<number, string>();
    const next = new Map<number, string>();
    const labels: string[] = [];
    for (const sample of samples()) {
      const label = previous.get(sample.ts) ?? formatter.format(sample.ts);
      next.set(sample.ts, label);
      labels.push(label);
    }
    cachedLocale = tag;
    cached = next;
    const size = limit();
    while (labels.length < size) labels.push("");
    return labels;
  });
}
