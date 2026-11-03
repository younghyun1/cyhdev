import type { UiLocale } from "../i18n/keys";

/**
 * Estimated reading time, never below one minute. Space-separated languages
 * count words at 220 per minute; Chinese and Japanese, which do not separate
 * words with spaces, count non-space characters at 500 per minute.
 */
export function estimateReadingMinutes(text: string, locale: UiLocale): number {
  if (locale === "ja-JP" || locale === "zh-Hans" || locale === "zh-Hant") {
    const characters = text.replace(/\s+/gu, "").length;
    return Math.max(1, Math.round(characters / 500));
  }
  const words = text.split(/\s+/u).filter(Boolean).length;
  return Math.max(1, Math.round(words / 220));
}

export type OutlineEntry = {
  readonly id: string;
  readonly text: string;
  readonly level: 2 | 3;
};

/** Longest generated fragment; headings longer than this are truncated. */
const MAX_SLUG_LENGTH = 64;

/**
 * Builds a fragment identifier from heading text. Letters and digits from
 * any script are kept, so Korean or Japanese headings still produce readable
 * fragments; everything else collapses to single hyphens.
 */
export function slugify(text: string): string {
  return text
    .normalize("NFKC")
    .toLowerCase()
    .replace(/[^\p{L}\p{N}]+/gu, "-")
    .replace(/^-+|-+$/gu, "")
    .slice(0, MAX_SLUG_LENGTH);
}

/**
 * Assigns `post-` prefixed ids to `h2` and `h3` headings that lack one and
 * returns the outline. The prefix keeps generated ids from clobbering global
 * names; duplicates receive numeric suffixes. Existing ids are kept.
 */
export function outlineHeadings(root: ParentNode): OutlineEntry[] {
  const used = new Set<string>();
  const entries: OutlineEntry[] = [];
  for (const heading of Array.from(root.querySelectorAll<HTMLElement>("h2, h3"))) {
    const text = heading.textContent?.trim() ?? "";
    if (!text) continue;
    let id = heading.id;
    if (!id) {
      const base = `post-${slugify(text) || "section"}`;
      id = base;
      for (let suffix = 2; used.has(id) || document.getElementById(id); suffix += 1) {
        id = `${base}-${suffix}`;
      }
      heading.id = id;
    }
    used.add(id);
    entries.push({ id, text, level: heading.tagName === "H2" ? 2 : 3 });
  }
  return entries;
}
