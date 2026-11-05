import { Show } from "solid-js";

import type { ForumAuthor } from "../../services/contracts/forum_types";

interface ForumAuthorProps {
  readonly author: ForumAuthor;
}

/** Round picture, or the name's first letter when the account has none. */
export function ForumAvatar(props: ForumAuthorProps & { readonly size?: "sm" | "lg" }) {
  return (
    <Show
      when={props.author.profile_picture_url}
      fallback={
        <span class="forum-avatar" data-size={props.size ?? "sm"} aria-hidden="true">
          {Array.from(props.author.display_name)[0] ?? "?"}
        </span>
      }
    >
      {(url) => (
        <img
          class="forum-avatar"
          data-size={props.size ?? "sm"}
          src={url()}
          alt=""
          width="40"
          height="40"
          loading="lazy"
          decoding="async"
        />
      )}
    </Show>
  );
}

/** Links to the public profile; tombstoned accounts have none to link to. */
export function ForumAuthorName(props: ForumAuthorProps) {
  return (
    <Show
      when={!props.author.is_deleted}
      fallback={<span class="forum-author-name" dir="auto">{props.author.display_name}</span>}
    >
      <a
        class="forum-author-name"
        dir="auto"
        href={`/users/${encodeURIComponent(props.author.display_name)}`}
      >
        {props.author.display_name}
      </a>
    </Show>
  );
}

export default function ForumAuthorBadge(props: ForumAuthorProps) {
  return (
    <span class="forum-author">
      <ForumAvatar author={props.author} />
      <ForumAuthorName author={props.author} />
    </span>
  );
}
