import { For, createMemo } from "solid-js";
import type { JSX } from "@solidjs/web";
import {
  parseInline,
  type InlineLabelSegment,
  type InlineSegment,
} from "../../utils/inlineMarkup";

function renderLabel(part: InlineLabelSegment): JSX.Element {
  return part.kind === "code" ? <code class="cp-code">{part.text}</code> : part.text;
}

function renderSegment(segment: InlineSegment): JSX.Element {
  switch (segment.kind) {
    case "text":
      return segment.text;
    case "strong":
      return <strong class="cp-strong">{segment.text}</strong>;
    case "code":
      return <code class="cp-code">{segment.text}</code>;
    case "link": {
      // mailto links open the mail client in place; other external links get
      // a new browsing context without an opener or referrer.
      const newContext = segment.external && !segment.href.startsWith("mailto:");
      return (
        <a
          href={segment.href}
          class="cp-link"
          target={newContext ? "_blank" : undefined}
          rel={segment.external ? "noopener noreferrer" : undefined}
        >
          <For each={segment.label}>{renderLabel}</For>
        </a>
      );
    }
  }
}

/** Renders translated copy written in the `utils/inlineMarkup.ts` syntax. */
export default function InlineText(props: { text: string }): JSX.Element {
  const segments = createMemo(() => parseInline(props.text));
  return <For each={segments()}>{renderSegment}</For>;
}
