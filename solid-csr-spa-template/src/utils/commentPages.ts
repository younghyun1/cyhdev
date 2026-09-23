// Accumulates keyset comment pages for threaded display.
//
// Comment endpoints return oldest-first pages. A reply is always newer than
// its parent, so once every loaded page is merged the whole thread can be
// rebuilt; a comment created locally may later arrive in a fetched page, and
// an edit or deletion made here replaces the fetched copy.

/**
 * Merges comment pages in order, keeping the first copy of each ID and
 * substituting any local replacement (an edit or tombstone made in this view).
 */
export function mergeCommentPages<T>(
  pages: ReadonlyArray<ReadonlyArray<T>>,
  idOf: (comment: T) => string,
  replacementFor: (id: string) => T | undefined,
): T[] {
  const seen = new Set<string>();
  const merged: T[] = [];
  for (const page of pages) {
    for (const comment of page) {
      const id = idOf(comment);
      if (seen.has(id)) continue;
      seen.add(id);
      merged.push(replacementFor(id) ?? comment);
    }
  }
  return merged;
}
