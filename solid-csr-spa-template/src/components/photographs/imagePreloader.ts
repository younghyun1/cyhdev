/**
 * Bounded, cancel-safe image preloading for the photograph viewer.
 *
 * Each entry is a detached image element whose `decode()` promise settles once
 * the file is fetched and decoded, so a later `<img>` with the same URL paints
 * without a blank frame. Entries beyond `maxEntries`, or dropped by `retain`,
 * have their request aborted by removing `src`; callers guard their own
 * completions, so an aborted or superseded load can never update the viewer.
 */

export type PreloadPriority = "high" | "low";

type EntryState = "pending" | "ready" | "failed";

interface Entry {
  readonly image: HTMLImageElement;
  readonly ready: Promise<void>;
  state: EntryState;
}

export interface ImagePreloader {
  /**
   * Resolves once `url` is decoded and rejects if it cannot be loaded. Pending
   * and decoded entries are shared; a previously failed URL is requested again.
   */
  load(url: string, priority?: PreloadPriority): Promise<void>;
  /** Starts a low-priority load unless `url` is already tracked, even as failed. */
  warm(url: string): void;
  /** Aborts and forgets every entry whose URL is not listed. */
  retain(urls: readonly string[]): void;
  /** Aborts and forgets everything; call when the viewer unmounts. */
  clear(): void;
  /** Number of tracked entries, for tests and diagnostics. */
  readonly size: number;
}

export interface ImagePreloaderOptions {
  /** Upper bound on tracked images; the least recently requested is evicted. */
  readonly maxEntries: number;
  /** Image factory, replaceable in tests. */
  readonly createImage?: () => HTMLImageElement;
}

/** Fetches and decodes `image.src`, falling back to load events without `decode()`. */
function decodeImage(image: HTMLImageElement): Promise<void> {
  if (typeof image.decode === "function") return image.decode();
  return new Promise<void>((resolve, reject) => {
    image.onload = () => resolve();
    image.onerror = () => reject(new Error("image failed to load"));
  });
}

export function createImagePreloader(options: ImagePreloaderOptions): ImagePreloader {
  const maxEntries = Math.max(1, options.maxEntries);
  const createImage = options.createImage ?? (() => new Image());
  // Map iteration order is insertion order; re-inserting on use makes the
  // first key the least recently requested entry.
  const entries = new Map<string, Entry>();

  const abort = (url: string) => {
    const entry = entries.get(url);
    if (!entry) return;
    entries.delete(url);
    entry.image.removeAttribute("src");
  };

  const start = (url: string, priority: PreloadPriority): Entry => {
    const image = createImage();
    image.decoding = "async";
    image.fetchPriority = priority;
    image.src = url;
    const ready = decodeImage(image);
    const entry: Entry = { image, ready, state: "pending" };
    // Attaching both handlers here also marks the rejection as handled for
    // warm-ups that nobody awaits.
    ready.then(
      () => {
        entry.state = "ready";
      },
      () => {
        entry.state = "failed";
      },
    );
    entries.set(url, entry);
    while (entries.size > maxEntries) {
      const oldest = entries.keys().next();
      if (oldest.done || oldest.value === url) break;
      abort(oldest.value);
    }
    return entry;
  };

  return {
    load(url, priority = "high") {
      const existing = entries.get(url);
      if (existing && existing.state !== "failed") {
        entries.delete(url);
        entries.set(url, existing);
        return existing.ready;
      }
      if (existing) abort(url);
      return start(url, priority).ready;
    },
    warm(url) {
      if (!entries.has(url)) start(url, "low");
    },
    retain(urls) {
      const keep = new Set(urls);
      for (const url of [...entries.keys()]) {
        if (!keep.has(url)) abort(url);
      }
    },
    clear() {
      for (const url of [...entries.keys()]) abort(url);
    },
    get size() {
      return entries.size;
    },
  };
}
