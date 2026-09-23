/**
 * Bounded chat history window.
 *
 * The panel keeps at most {@link LIVE_CHAT_HISTORY_LIMIT} entries and always
 * trims the end farther from what the reader is looking at:
 *
 * - At the live edge (scrolled to the bottom), new messages append and the
 *   oldest entries drop.
 * - Paging back prepends older messages and drops the newest. The window then
 *   no longer reaches the live tail and is "detached".
 * - While the reader is scrolled into history, live messages append only
 *   while there is room; a full window is never trimmed under the reader, so
 *   the message instead marks the window detached and waits in a small buffer.
 *
 * A detached window resynchronizes with the latest page when the reader
 * returns to the bottom or sends a message.
 */

export const LIVE_CHAT_HISTORY_LIMIT = 300;
/** Live messages held while detached; the latest page covers anything older. */
export const LIVE_CHAT_DETACHED_BUFFER_LIMIT = 50;

export interface WindowUpdate<T> {
  list: T[];
  /** The window no longer contains the newest live messages. */
  detached: boolean;
  /** Older entries were dropped, so history before the window can be paged again. */
  trimmedOldest: boolean;
}

/** Add a live message. `atLiveEdge` means the reader is at the bottom. */
export function appendLive<T>(
  list: readonly T[],
  item: T,
  atLiveEdge: boolean,
  limit = LIVE_CHAT_HISTORY_LIMIT,
): WindowUpdate<T> {
  if (list.length < limit) {
    return { list: [...list, item], detached: false, trimmedOldest: false };
  }
  if (!atLiveEdge) {
    return { list: [...list], detached: true, trimmedOldest: false };
  }
  return {
    list: [...list, item].slice(-limit),
    detached: false,
    trimmedOldest: true,
  };
}

/** Bound a list that grew at the live edge, dropping its oldest entries. */
export function trimOldest<T>(
  list: readonly T[],
  limit = LIVE_CHAT_HISTORY_LIMIT,
): WindowUpdate<T> {
  if (list.length <= limit) {
    return { list: [...list], detached: false, trimmedOldest: false };
  }
  return { list: list.slice(-limit), detached: false, trimmedOldest: true };
}

/** Add an older page in front, dropping the newest entries beyond the limit. */
export function prependOlder<T>(
  list: readonly T[],
  older: readonly T[],
  limit = LIVE_CHAT_HISTORY_LIMIT,
): WindowUpdate<T> {
  const combined = [...older, ...list];
  if (combined.length <= limit) {
    return { list: combined, detached: false, trimmedOldest: false };
  }
  return { list: combined.slice(0, limit), detached: true, trimmedOldest: false };
}

/** Remember a live message that arrived while detached, keeping the newest. */
export function bufferDetached<T>(
  buffer: readonly T[],
  item: T,
  limit = LIVE_CHAT_DETACHED_BUFFER_LIMIT,
): T[] {
  return [...buffer, item].slice(-limit);
}

/**
 * Rebuild the window from the latest page plus messages buffered while
 * detached: deduplicated by id, ordered by `(created, id)`, then bounded.
 */
export function mergeLatest<T>(
  latest: readonly T[],
  buffered: readonly T[],
  identify: (item: T) => { id: string; createdAt: string },
  limit = LIVE_CHAT_HISTORY_LIMIT,
): T[] {
  const byId = new Map<string, T>();
  for (const item of [...latest, ...buffered]) {
    const { id } = identify(item);
    if (!byId.has(id)) byId.set(id, item);
  }
  return [...byId.values()]
    .sort((left, right) => {
      const a = identify(left);
      const b = identify(right);
      const byTime = Date.parse(a.createdAt) - Date.parse(b.createdAt);
      if (byTime !== 0) return byTime;
      return a.id < b.id ? -1 : a.id > b.id ? 1 : 0;
    })
    .slice(-limit);
}
