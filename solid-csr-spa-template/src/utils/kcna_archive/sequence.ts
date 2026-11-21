// Request sequencing: which responses may still be shown.

import { PAGE_SIZE, RESULT_WINDOW } from "./constants";
import type { FailureKind } from "./failure";

/**
 * Numbers requests so only the newest one's response is applied. A response
 * for a query the reader has already left must not replace what is on screen.
 */
export interface RequestGate {
  /** Starts a request and returns its ticket; older tickets stop being current. */
  readonly begin: () => number;
  /** Whether no newer request, and no invalidation, happened since `ticket`. */
  readonly isCurrent: (ticket: number) => boolean;
  /** Makes every outstanding ticket stale without starting a request. */
  readonly invalidate: () => void;
}

export const createRequestGate = (): RequestGate => {
  let latest = 0;
  return {
    begin: () => {
      latest += 1;
      return latest;
    },
    isCurrent: (ticket) => ticket === latest,
    invalidate: () => {
      latest += 1;
    },
  };
};

/** The result pages after the first, which the list loads as the reader scrolls. */
export interface LaterPages<T> {
  /** The query these pages belong to. */
  readonly key: string;
  readonly items: ReadonlyArray<T>;
  readonly loading: boolean;
  readonly failure: FailureKind | undefined;
}

export const noLaterPages = <T>(): LaterPages<T> => ({
  key: "",
  items: [],
  loading: false,
  failure: undefined,
});

/** The pages that belong to `key`; a state left over from another query is empty. */
export const pagesFor = <T>(state: LaterPages<T>, key: string): LaterPages<T> =>
  state.key === key ? state : { ...noLaterPages<T>(), key };

/** Where the next page starts, or `undefined` when the server's result window is used up. */
export const nextOffset = (state: LaterPages<unknown>): number | undefined => {
  const offset = PAGE_SIZE + state.items.length;
  return offset >= RESULT_WINDOW ? undefined : offset;
};

export const pageRequested = <T>(state: LaterPages<T>, key: string): LaterPages<T> => ({
  ...pagesFor(state, key),
  loading: true,
  failure: undefined,
});

/** Appends a page, skipping articles already shown, within the result window. */
export const pageReceived = <T extends { readonly id: string }>(
  state: LaterPages<T>,
  key: string,
  page: ReadonlyArray<T>,
): LaterPages<T> => {
  const current = pagesFor(state, key);
  const known = new Set(current.items.map((item) => item.id));
  const room = Math.max(0, RESULT_WINDOW - PAGE_SIZE - current.items.length);
  const fresh = page.filter((item) => !known.has(item.id)).slice(0, room);
  return { key, items: [...current.items, ...fresh], loading: false, failure: undefined };
};

export const pageFailed = <T>(
  state: LaterPages<T>,
  key: string,
  failure: FailureKind,
): LaterPages<T> => ({ ...pagesFor(state, key), loading: false, failure });
