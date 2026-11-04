import { Show } from "solid-js";
import type { JSX } from "@solidjs/web";

import type {
  ForumAuthor,
  ForumContentState,
} from "../../services/contracts/forum_types";
import { t, tx } from "../../state/i18n";
import { ForumAuthorName, ForumAvatar } from "./ForumAuthor";
import ForumTime from "./ForumTime";

interface ForumPostProps {
  readonly id: string;
  /** Position in the thread: the opening post is 1, replies count up from 2. */
  readonly number: number;
  readonly author: ForumAuthor;
  readonly isOriginalPoster: boolean;
  readonly createdAt: string;
  readonly editedAt: string | null;
  readonly contentState: ForumContentState;
  readonly actions?: JSX.Element;
  readonly children: JSX.Element;
}

/**
 * One post in a thread, in the classic forum shape: an author column beside
 * the text, a numbered permalink, and actions underneath. Phones fold the
 * author column into a row above the text.
 */
export default function ForumPost(props: ForumPostProps) {
  return (
    <article
      id={props.id}
      class="forum-post"
      data-masked={props.contentState !== "visible" ? "" : undefined}
    >
      <aside class="forum-post__author">
        <ForumAvatar author={props.author} size="lg" />
        <div class="forum-post__author-text">
          <ForumAuthorName author={props.author} />
          <Show when={props.isOriginalPoster}>
            <span class="forum-tag" title={t("forum.post.original_poster_title")}>
              {t("forum.post.original_poster")}
            </span>
          </Show>
        </div>
      </aside>
      <div class="forum-post__main">
        <header class="forum-post__head">
          <ForumTime at={props.createdAt} />
          <Show when={props.editedAt}>
            <span>{t("forum.topic.edited")}</span>
          </Show>
          <Show when={props.contentState === "hidden"}>
            <span class="forum-badge forum-badge--danger">{t("forum.topic.hidden")}</span>
          </Show>
          <Show when={props.contentState === "deleted"}>
            <span class="forum-badge">{t("forum.topic.deleted")}</span>
          </Show>
          <a
            class="forum-post__number"
            href={`#${props.id}`}
            aria-label={tx("forum.post.permalink", { number: props.number })}
          >
            #{props.number}
          </a>
        </header>
        <div class="forum-post__content">{props.children}</div>
        <Show when={props.actions}>
          <footer class="forum-post__actions">{props.actions}</footer>
        </Show>
      </div>
    </article>
  );
}
