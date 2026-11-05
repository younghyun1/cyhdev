import { For, createMemo } from "solid-js";

import { type ForumInline, parseForumBody } from "../../utils/forumBody";

function Inline(props: { readonly parts: ReadonlyArray<ForumInline> }) {
  return (
    <For each={props.parts}>
      {(part) =>
        part.kind === "link" ? (
          <a href={part.href} dir="ltr" target="_blank" rel="nofollow noopener noreferrer ugc">
            {part.text}
          </a>
        ) : (
          part.text
        )
      }
    </For>
  );
}

/**
 * Splits inline parts at newlines. dir=auto reads only the first strong
 * character of an element, so a post that mixes an English line and an Arabic
 * line needs each line to be its own element to get its own direction.
 */
function splitLines(parts: ReadonlyArray<ForumInline>): ReadonlyArray<ReadonlyArray<ForumInline>> {
  const lines: ForumInline[][] = [];
  let current: ForumInline[] = [];
  lines.push(current);
  for (const part of parts) {
    if (part.kind === "link") {
      current.push(part);
      continue;
    }
    part.text.split("\n").forEach((piece, index) => {
      if (index > 0) {
        current = [];
        lines.push(current);
      }
      if (piece) current.push({ kind: "text", text: piece });
    });
  }
  return lines;
}

function Lines(props: { readonly parts: ReadonlyArray<ForumInline> }) {
  const lines = createMemo(() => splitLines(props.parts));
  return (
    <For each={lines()}>
      {(line) => (
        <div class="forum-line" dir="auto">
          <Inline parts={line} />
        </div>
      )}
    </For>
  );
}

/** Renders a plain-text body as text nodes with quote blocks and URL links. */
export default function ForumPostBody(props: { readonly body: string }) {
  const blocks = createMemo(() => parseForumBody(props.body));
  return (
    <div class="forum-body">
      <For each={blocks()}>
        {(block) =>
          block.kind === "quote" ? (
            <blockquote class="forum-quote" dir="auto">
              <Lines parts={block.inline} />
            </blockquote>
          ) : (
            <div class="forum-text">
              <Lines parts={block.inline} />
            </div>
          )
        }
      </For>
    </div>
  );
}
