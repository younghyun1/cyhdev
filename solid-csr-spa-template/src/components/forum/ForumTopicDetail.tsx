import { Show, createSignal } from "solid-js";

import type {
  ForumTopic,
  ForumTopicModerationAction,
} from "../../services/contracts/forum_types";
import { t, tx } from "../../state/i18n";
import { ForumAuthorName } from "./ForumAuthor";
import ForumModerationControls from "./ForumModerationControls";
import ForumPost from "./ForumPost";
import ForumPostBody from "./ForumPostBody";
import ForumTime from "./ForumTime";
import ForumTopicForm from "./ForumTopicForm";

interface ForumTopicDetailProps {
  readonly topic: ForumTopic;
  readonly own: boolean;
  readonly authenticated: boolean;
  readonly canModerate: boolean;
  readonly subscribed: boolean;
  /** Present when the viewer can reply to this topic. */
  readonly onReply?: () => void;
  readonly onQuote?: () => void;
  readonly onUpdate: (title: string, body: string, revision: number) => Promise<void>;
  readonly onDelete: (revision: number) => Promise<void>;
  readonly onSubscription: (subscribe: boolean) => Promise<void>;
  readonly onModerate: (
    action: ForumTopicModerationAction,
    reason: string,
    revision: number,
  ) => Promise<void>;
}

/** Topic heading and toolbar, followed by the opening post of the thread. */
export default function ForumTopicDetailCard(props: ForumTopicDetailProps) {
  const [editing, setEditing] = createSignal(false);
  const [busy, setBusy] = createSignal(false);
  const [subscriptionBusy, setSubscriptionBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

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
    <>
      <header class="forum-topic-header">
        <div class="forum-topic-header__text">
          <div class="forum-badges">
            <Show when={props.topic.is_pinned}>
              <span class="forum-badge forum-badge--accent">
                {t("forum.topic.pinned")}
              </span>
            </Show>
            <Show when={props.topic.access_state === "locked"}>
              <span class="forum-badge">{t("forum.topic.locked")}</span>
            </Show>
          </div>
          <h1 class="forum-topic-header__title" dir="auto">
            {props.topic.title ?? t("forum.topic.masked")}
          </h1>
          <p class="forum-topic-header__meta">
            <ForumAuthorName author={props.topic.author} />
            <span aria-hidden="true"> · </span>
            <ForumTime at={props.topic.created_at} />
            <span aria-hidden="true"> · </span>
            {tx("forum.topic.replies", { count: props.topic.reply_count })}
          </p>
        </div>
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
          <Show when={props.onReply}>
            <button
              class="forum-button forum-button--primary"
              type="button"
              onClick={() => props.onReply?.()}
            >
              {t("forum.topic.reply_action")}
            </button>
          </Show>
        </div>
      </header>
      <Show when={error()}>
        {(message) => (
          <div class="forum-alert forum-alert--error" role="alert">
            {message()}
          </div>
        )}
      </Show>
      <ForumPost
        id={`forum-topic-${props.topic.topic_id}`}
        number={1}
        author={props.topic.author}
        isOriginalPoster={true}
        createdAt={props.topic.created_at}
        editedAt={props.topic.edited_at}
        contentState={props.topic.content_state}
        actions={
          <>
            <Show when={props.onQuote && props.topic.content_state === "visible"}>
              <button class="forum-text-button" type="button" onClick={() => props.onQuote?.()}>
                {t("forum.post.quote")}
              </button>
            </Show>
            <Show when={props.own && props.topic.content_state !== "deleted"}>
              <Show when={props.topic.content_state === "visible"}>
                <button
                  class="forum-text-button"
                  type="button"
                  disabled={busy()}
                  onClick={() => setEditing(true)}
                >
                  {t("forum.topic.edit")}
                </button>
              </Show>
              <button
                class="forum-text-button forum-text-button--danger"
                type="button"
                disabled={busy()}
                onClick={() => void remove()}
              >
                {t("forum.topic.delete")}
              </button>
            </Show>
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
          </>
        }
      >
        <Show
          when={editing()}
          fallback={<ForumPostBody body={props.topic.body ?? t("forum.topic.masked")} />}
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
      </ForumPost>
    </>
  );
}
