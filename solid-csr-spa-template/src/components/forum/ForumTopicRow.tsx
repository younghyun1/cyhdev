import { Show } from "solid-js";

import type { ForumTopic } from "../../services/contracts/forum_types";
import { locale, t, tx } from "../../state/i18n";
import { ForumAuthorName, ForumAvatar } from "./ForumAuthor";
import ForumTime from "./ForumTime";

interface ForumTopicRowProps {
  readonly topic: ForumTopic;
}

/** One row of the topic table; the title link covers the whole row. */
export default function ForumTopicRow(props: ForumTopicRowProps) {
  const masked = () => props.topic.content_state !== "visible";
  const title = () => props.topic.title ?? t("forum.topic.masked");

  return (
    <article class="forum-topic-row" data-masked={masked() ? "" : undefined}>
      <div class="forum-topic-row__topic">
        <ForumAvatar author={props.topic.author} />
        <div class="forum-topic-row__text">
          <h3 class="forum-topic-row__title">
            <Show when={props.topic.is_pinned}>
              <span class="forum-badge forum-badge--accent">{t("forum.topic.pinned")}</span>
            </Show>
            <Show when={props.topic.access_state === "locked"}>
              <span class="forum-badge">{t("forum.topic.locked")}</span>
            </Show>
            <Show when={props.topic.content_state === "hidden"}>
              <span class="forum-badge forum-badge--danger">{t("forum.topic.hidden")}</span>
            </Show>
            <Show when={props.topic.content_state === "deleted"}>
              <span class="forum-badge">{t("forum.topic.deleted")}</span>
            </Show>
            <a
              class="forum-topic-row__link"
              dir="auto"
              href={`/forum/${encodeURIComponent(props.topic.topic_id)}`}
            >
              {title()}
            </a>
          </h3>
          <Show when={!masked() && props.topic.body}>
            {(value) => <p class="forum-topic-row__preview" dir="auto">{value()}</p>}
          </Show>
          <p class="forum-topic-row__byline">
            <ForumAuthorName author={props.topic.author} />
            <span aria-hidden="true"> · </span>
            <ForumTime at={props.topic.created_at} />
          </p>
        </div>
      </div>
      <p class="forum-topic-row__replies">
        <strong>{props.topic.reply_count}</strong>
        <span class="forum-topic-row__replies-label">{t("forum.table.replies")}</span>
      </p>
      <p class="forum-topic-row__activity">
        <span class="sr-only">
          {tx("forum.topic.activity", {
            date: new Date(props.topic.last_activity_at).toLocaleDateString(locale()),
          })}
        </span>
        <span aria-hidden="true">
          <ForumTime at={props.topic.last_activity_at} />
        </span>
      </p>
    </article>
  );
}
