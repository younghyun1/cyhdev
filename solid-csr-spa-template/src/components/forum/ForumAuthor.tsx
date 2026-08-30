import { Show } from "solid-js";

import type { ForumAuthor } from "../../services/contracts/forum_types";

interface ForumAuthorProps {
  readonly author: ForumAuthor;
}

export default function ForumAuthorBadge(props: ForumAuthorProps) {
  return (
    <span class="forum-row forum-meta">
      <Show when={props.author.profile_picture_url}>
        {(url) => (
          <img
            class="forum-author__avatar"
            src={url()}
            alt=""
            width="24"
            height="24"
          />
        )}
      </Show>
      <span>{props.author.display_name}</span>
    </span>
  );
}
