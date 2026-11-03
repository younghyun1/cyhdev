/**
 * Minimal inline markup for translated page copy: `**strong**`, `` `code` ``,
 * and `[label](href)`. Translators edit plain strings while the renderer emits
 * real elements, so no page copy ever reaches `innerHTML`. Markers do not nest
 * except that a link label may contain code. Unmatched markers stay literal
 * text, and only `https:`, `mailto:`, and same-origin `/` links are accepted;
 * any other target renders as plain text.
 */
export type InlineSegment =
  | { readonly kind: "text"; readonly text: string }
  | { readonly kind: "strong"; readonly text: string }
  | { readonly kind: "code"; readonly text: string }
  | {
      readonly kind: "link";
      readonly href: string;
      readonly external: boolean;
      readonly label: ReadonlyArray<InlineLabelSegment>;
    };

export type InlineLabelSegment =
  | { readonly kind: "text"; readonly text: string }
  | { readonly kind: "code"; readonly text: string };

/** Longest accepted input; page copy is static, so this only guards mistakes. */
const MAX_INLINE_LENGTH = 8192;

export function isSafeHref(href: string): boolean {
  if (href.startsWith("/") && !href.startsWith("//")) return true;
  return /^https:\/\/[^\s]+$/u.test(href) || /^mailto:[^\s]+$/u.test(href);
}

function parseLabel(label: string): InlineLabelSegment[] {
  const segments: InlineLabelSegment[] = [];
  let rest = label;
  while (rest.length > 0) {
    const open = rest.indexOf("`");
    const close = open >= 0 ? rest.indexOf("`", open + 1) : -1;
    if (open < 0 || close < 0) {
      segments.push({ kind: "text", text: rest });
      break;
    }
    if (open > 0) segments.push({ kind: "text", text: rest.slice(0, open) });
    segments.push({ kind: "code", text: rest.slice(open + 1, close) });
    rest = rest.slice(close + 1);
  }
  return segments;
}

export function parseInline(source: string): InlineSegment[] {
  const input =
    source.length > MAX_INLINE_LENGTH ? source.slice(0, MAX_INLINE_LENGTH) : source;
  const segments: InlineSegment[] = [];
  let text = "";
  const flushText = () => {
    if (text.length > 0) segments.push({ kind: "text", text });
    text = "";
  };

  let index = 0;
  while (index < input.length) {
    const char = input[index];
    if (char === "*" && input[index + 1] === "*") {
      const end = input.indexOf("**", index + 2);
      if (end > index + 2) {
        flushText();
        segments.push({ kind: "strong", text: input.slice(index + 2, end) });
        index = end + 2;
        continue;
      }
    } else if (char === "`") {
      const end = input.indexOf("`", index + 1);
      if (end > index + 1) {
        flushText();
        segments.push({ kind: "code", text: input.slice(index + 1, end) });
        index = end + 1;
        continue;
      }
    } else if (char === "[") {
      const labelEnd = input.indexOf("](", index + 1);
      const hrefEnd = labelEnd >= 0 ? input.indexOf(")", labelEnd + 2) : -1;
      if (labelEnd > index + 1 && hrefEnd > labelEnd + 2) {
        const label = input.slice(index + 1, labelEnd);
        const href = input.slice(labelEnd + 2, hrefEnd);
        flushText();
        if (isSafeHref(href)) {
          segments.push({
            kind: "link",
            href,
            external: !href.startsWith("/"),
            label: parseLabel(label),
          });
        } else {
          segments.push({ kind: "text", text: label });
        }
        index = hrefEnd + 1;
        continue;
      }
    }
    text += char;
    index += 1;
  }
  flushText();
  return segments;
}

/** Plain-text projection used for search indexes and accessible summaries. */
export function inlineToPlainText(source: string): string {
  return parseInline(source)
    .map((segment) =>
      segment.kind === "link"
        ? segment.label.map((part) => part.text).join("")
        : segment.text,
    )
    .join("");
}
