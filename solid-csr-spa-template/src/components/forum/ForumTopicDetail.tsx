import { Show, createSignal } from "solid-js";

import type {
  ForumTopic,
  ForumTopicModerationAction,
} from "../../services/contracts/forum_types";
import { locale, t, tx } from "../../state/i18n";
import ForumAuthorBadge from "./ForumAuthor";
import ForumModerationControls from "./ForumModerationControls";
import ForumTopicForm from "./ForumTopicForm";

interface ForumTopicDetailProps {
  readonly topic: ForumTopic;
  readonly own: boolean;
  readonly authenticated: boolean;
  readonly canModerate: boolean;
  readonly subscribed: boolean;
  readonly onUpdate: (title: string, body: string, revision: number) => Promise<void>;
  readonly onDelete: (revision: number) => Promise<void>;
  readonly onSubscription: (subscribe: boolean) => Promise<void>;
  readonly onModerate: (
    action: ForumTopicModerationAction,
    reason: string,
    revision: number,
  ) => Promise<void>;
}

export default function ForumTopicDetailCard(props: ForumTopicDetailProps) {
  const [editing, setEditing] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [subscriptionBusy, setSubscriptionBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const created = () => new Date(props.topic.created_at).toLocaleString(locale());

  const update = async (title: string, body: string): Promise<void> => {
    setBusy(true);
    setError(null);
    try {
      await props.onUpdate(title, body, props.topic.revision);
      setEditing(false);
    } catch {
      setError(t("forum.topic.update_failed"));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (): Promise<void> => {
    if (!confirm(t("forum.topic.delete_confirm"))) return;
    setBusy(true);
    setError(null);
    try {
      await props.onDelete(props.topic.revision);
    } catch {
      setError(t("forum.topic.delete_failed"));
    } finally {
      setBusy(false);
    }
  };

  const toggleSubscription = async (): Promise<void> => {
    setSubscriptionBusy(true);
    setError(null);
    try {
      await props.onSubscription(!props.subscribed);
    } catch {
      setError(t("forum.topic.subscription_failed"));
    } finally {
      setSubscriptionBusy(false);
    }
  };

  const moderationActions = (): ReadonlyArray<ForumTopicModerationAction> => {
    const actions: ForumTopicModerationAction[] = [];
    if (props.topic.content_state === "visible") actions.push("hide");
    if (props.topic.content_state === "hidden") actions.push("restore");
    actions.push(props.topic.access_state === "locked" ? "unlock" : "lock");
    actions.push(props.topic.is_pinned ? "unpin" : "pin");
    return actions;
  };

  return (
    <article class="forum-topic-detail">
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
      <Show
        when={editing()}
        fallback={
          <>
            <h1 class="forum-topic-detail__title">
              {props.topic.title ?? t("forum.topic.masked")}
            </h1>
            <div class="forum-topic-detail__meta">
              <ForumAuthorBadge author={props.topic.author} />
              <span>{tx("forum.topic.created", { date: created() })}</span>
              <Show when={props.topic.edited_at}>
                <span>{t("forum.topic.edited")}</span>
              </Show>
            </div>
            <p class="forum-topic-detail__body">
              {props.topic.body ?? t("forum.topic.masked")}
            </p>
          </>
        }
      >
        <ForumTopicForm
          idPrefix={`forum-topic-${props.topic.topic_id}`}
          initialTitle={props.topic.title ?? ""}
          initialBody={props.topic.body ?? ""}
          busy={busy()}
          mode="update"
          onSubmit={update}
          onCancel={() => setEditing(false)}
        />
      </Show>
      <Show when={error()}>
        {(message) => (
          <div class="forum-alert forum-alert--error" role="alert">
            {message()}
          </div>
        )}
      </Show>
      <div class="forum-actions">
        <Show when={props.authenticated && props.topic.content_state !== "deleted"}>
          <button
            class="forum-button"
            type="button"
            disabled={subscriptionBusy()}
            onClick={() => void toggleSubscription()}
          >
            {subscriptionBusy()
              ? t("forum.topic.subscription_pending")
              : props.subscribed
                ? t("forum.topic.unsubscribe")
                : t("forum.topic.subscribe")}
          </button>
        </Show>
        <Show when={props.own && props.topic.content_state !== "deleted"}>
          <Show when={props.topic.content_state === "visible"}>
            <button
              class="forum-button"
              type="button"
              disabled={busy()}
              onClick={() => setEditing(true)}
            >
              {t("forum.topic.edit")}
            </button>
          </Show>
          <button
            class="forum-button forum-button--danger"
            type="button"
            disabled={busy()}
            onClick={() => void remove()}
          >
            {t("forum.topic.delete")}
          </button>
        </Show>
      </div>
      <Show when={props.canModerate && props.topic.content_state !== "deleted"}>
        <ForumModerationControls
          idPrefix={`forum-topic-${props.topic.topic_id}-moderation`}
          actions={moderationActions()}
          onModerate={(action, reason) =>
            props.onModerate(
              action as ForumTopicModerationAction,
              reason,
              props.topic.revision,
            )
          }
        />
      </Show>
    </article>
  );
}
