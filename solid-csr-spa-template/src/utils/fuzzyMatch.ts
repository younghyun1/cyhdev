/**
 * Ranks `text` against a palette query; lower scores are better and `null`
 * means no match. A contiguous substring wins, earlier positions and word
 * starts winning ties; otherwise the query's characters must appear in order,
 * and wider gaps cost more. Comparison is case-insensitive and ignores
 * diacritics, so "uber" finds "Über". Scripts without letter case, such as
 * Hangul and Han characters, match on the same substring and order rules.
 */
export function normalizeForMatch(value: string): string {
  return value.normalize("NFKD").replace(/\p{M}+/gu, "").toLowerCase();
}

export function fuzzyScore(query: string, text: string): number | null {
  const needle = normalizeForMatch(query.trim());
  if (needle.length === 0) return 0;
  const haystack = normalizeForMatch(text);

  const index = haystack.indexOf(needle);
  if (index >= 0) {
    const atWordStart = index === 0 || /[\s/›·\-_.]/u.test(haystack[index - 1] ?? "");
    return (atWordStart ? 0 : 10) + index;
  }

  let position = -1;
  let gaps = 0;
  for (const char of needle) {
    if (char === " ") continue;
    const next = haystack.indexOf(char, position + 1);
    if (next < 0) return null;
    if (position >= 0) gaps += next - position - 1;
    position = next;
  }
  return 100 + gaps;
}

/** Stable filter-and-sort of entries by their best-matching field. */
export function rankByQuery<T>(
  entries: ReadonlyArray<T>,
  query: string,
  fields: (entry: T) => ReadonlyArray<string>,
): T[] {
  const scored: Array<{ entry: T; score: number; order: number }> = [];
  entries.forEach((entry, order) => {
    let best: number | null = null;
    for (const field of fields(entry)) {
      const score = fuzzyScore(query, field);
      if (score !== null && (best === null || score < best)) best = score;
    }
    if (best !== null) scored.push({ entry, score: best, order });
  });
  scored.sort((a, b) => a.score - b.score || a.order - b.order);
  return scored.map((item) => item.entry);
}
