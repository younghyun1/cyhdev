import { Show } from "solid-js";

import type { ForumTopic } from "../../services/contracts/forum_types";
import { locale, t, tx } from "../../state/i18n";
import ForumAuthorBadge from "./ForumAuthor";

interface ForumTopicCardProps {
  readonly topic: ForumTopic;
}

const preview = (body: string): string => {
  const characters = Array.from(body);
  return characters.length <= 300
    ? body
    : `${characters.slice(0, 300).join("")}…`;
};

export default function ForumTopicCard(props: ForumTopicCardProps) {
  const masked = () => props.topic.content_state !== "visible";
  const title = () => props.topic.title ?? t("forum.topic.masked");
  const body = () => props.topic.body;
  const activity = () =>
    new Date(props.topic.last_activity_at).toLocaleString(locale());

  return (
    <article
      class={`forum-topic-card ${masked() ? "forum-topic-card--masked" : ""}`}
    >
      <div class="forum-badges">
        <Show when={props.topic.is_pinned}>
          <span class="forum-badge forum-badge--accent">
            {t("forum.topic.pinned")}
          </span>
        </Show>
        <Show when={props.topic.access_state === "locked"}>
          <span class="forum-badge">{t("forum.topic.locked")}</span>
        </Show>
        <Show when={props.topic.content_state === "hidden"}>
          <span class="forum-badge forum-badge--danger">
            {t("forum.topic.hidden")}
          </span>
        </Show>
        <Show when={props.topic.content_state === "deleted"}>
          <span class="forum-badge">{t("forum.topic.deleted")}</span>
        </Show>
      </div>
      <h2 class="forum-topic-card__title">
        <a href={`/forum/${encodeURIComponent(props.topic.topic_id)}`}>
          {title()}
        </a>
      </h2>
      <Show when={body()}>
        {(value) => <p class="forum-topic-card__preview">{preview(value())}</p>}
      </Show>
      <footer class="forum-topic-card__footer">
        <ForumAuthorBadge author={props.topic.author} />
        <span class="forum-meta">
          {tx("forum.topic.replies", { count: props.topic.reply_count })}
          {" · "}
          {tx("forum.topic.activity", { date: activity() })}
        </span>
      </footer>
    </article>
  );
}
