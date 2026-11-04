import { For, createMemo } from "solid-js";

import { type ForumInline, parseForumBody } from "../../utils/forumBody";

function Inline(props: { readonly parts: ReadonlyArray<ForumInline> }) {
  return (
    <For each={props.parts}>
      {(part) =>
        part.kind === "link" ? (
          <a href={part.href} target="_blank" rel="nofollow noopener noreferrer ugc">
            {part.text}
          </a>
        ) : (
          part.text
        )
      }
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
            <blockquote class="forum-quote">
              <Inline parts={block.inline} />
            </blockquote>
          ) : (
            <p>
              <Inline parts={block.inline} />
            </p>
          )
        }
      </For>
    </div>
  );
}
