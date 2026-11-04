/**
 * Plain-text forum bodies split for display. Lines starting with `>` become
 * quote blocks and bare http(s) URLs become links. The result renders as JSX
 * text nodes, never as HTML, and no other markup has meaning.
 */
export type ForumInline =
  | { readonly kind: "text"; readonly text: string }
  | { readonly kind: "link"; readonly text: string; readonly href: string };

export interface ForumBlock {
  readonly kind: "text" | "quote";
  readonly inline: ReadonlyArray<ForumInline>;
}

const URL_PATTERN = /https?:\/\/[^\s<>"'`]+/giu;
const TRAILING_PUNCTUATION = new Set(".,;:!?)]}");
/** Quoted text is capped so repeated quoting cannot use up a 20,000-character reply. */
const MAX_QUOTED_CHARACTERS = 2_000;

/**
 * Scans rather than using a `+$` regex, which is quadratic on a long punctuation
 * run in the middle of a URL. Anyone can post one, and it runs on every render.
 */
function withoutTrailingPunctuation(url: string): string {
  let end = url.length;
  while (end > 0 && TRAILING_PUNCTUATION.has(url.charAt(end - 1))) end -= 1;
  return url.slice(0, end);
}

function parseInline(text: string): ReadonlyArray<ForumInline> {
  const parts: ForumInline[] = [];
  let cursor = 0;
  for (const match of text.matchAll(URL_PATTERN)) {
    const trimmed = withoutTrailingPunctuation(match[0]);
    if (!URL.canParse(trimmed)) continue;
    if (match.index > cursor) {
      parts.push({ kind: "text", text: text.slice(cursor, match.index) });
    }
    parts.push({ kind: "link", text: trimmed, href: trimmed });
    cursor = match.index + trimmed.length;
  }
  if (cursor < text.length) parts.push({ kind: "text", text: text.slice(cursor) });
  return parts;
}

/** Groups consecutive lines into text and quote blocks. */
export function parseForumBody(body: string): ReadonlyArray<ForumBlock> {
  const blocks: ForumBlock[] = [];
  let lines: string[] = [];
  let kind: ForumBlock["kind"] = "text";
  const flush = () => {
    // Dropped by index for the same reason as above: long blank runs are free to post.
    let start = 0;
    let end = lines.length;
    while (start < end && lines[start] === "") start += 1;
    while (end > start && lines[end - 1] === "") end -= 1;
    const text = lines.slice(start, end).join("\n");
    if (text) blocks.push({ kind, inline: parseInline(text) });
    lines = [];
  };
  for (const line of body.split(/\r?\n/u)) {
    const quoted = line.startsWith(">");
    const lineKind = quoted ? "quote" : "text";
    if (lineKind !== kind) {
      flush();
      kind = lineKind;
    }
    lines.push(quoted ? line.replace(/^>\s?/u, "") : line);
  }
  flush();
  return blocks;
}

/**
 * Reply text quoting `body`. Quotes it already contains are dropped so
 * replies do not build pyramids of earlier posts.
 */
export function quoteForReply(attribution: string, body: string): string {
  const own = body
    .split(/\r?\n/u)
    .filter((line) => !line.startsWith(">"))
    .join("\n")
    .trim();
  const characters = Array.from(own);
  const clipped =
    characters.length > MAX_QUOTED_CHARACTERS
      ? `${characters.slice(0, MAX_QUOTED_CHARACTERS).join("")}…`
      : own;
  const lines = clipped.split("\n").map((line) => `> ${line}`.trimEnd());
  return `> ${attribution}\n${lines.join("\n")}\n\n`;
}
