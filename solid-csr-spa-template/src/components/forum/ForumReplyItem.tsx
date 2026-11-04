import { Show, createSignal, untrack } from "solid-js";

import type {
  ForumReply,
  ForumReplyModerationAction,
} from "../../services/contracts/forum_types";
import { t } from "../../state/i18n";
import ForumModerationControls from "./ForumModerationControls";
import ForumPost from "./ForumPost";
import ForumPostBody from "./ForumPostBody";

const encoder = new TextEncoder();

interface ForumReplyItemProps {
  readonly reply: ForumReply;
  /** Position in the thread; the opening post is 1. */
  readonly number: number;
  readonly isOriginalPoster: boolean;
  readonly own: boolean;
  readonly canModerate: boolean;
  /** Present when the viewer can reply to this topic. */
  readonly onQuote?: () => void;
  readonly onUpdate: (body: string, revision: number) => Promise<void>;
  readonly onDelete: (revision: number) => Promise<void>;
  readonly onModerate: (
    action: ForumReplyModerationAction,
    reason: string,
    revision: number,
  ) => Promise<void>;
}

export default function ForumReplyItem(props: ForumReplyItemProps) {
  const [editing, setEditing] = createSignal(false);
  const [body, setBody] = createSignal(
    untrack(() => props.reply.body ?? ""),
  );
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const update = async (event: SubmitEvent): Promise<void> => {
    event.preventDefault();
    const next = body().trim();
    if (
      !next ||
      Array.from(next).length > 20_000 ||
      encoder.encode(next).byteLength > 65_536
    ) {
      setError(t("forum.form.body_invalid"));
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await props.onUpdate(next, props.reply.revision);
      setEditing(false);
    } catch {
      setError(t("forum.reply.update_failed"));
    } finally {
      setBusy(false);
    }
  };

  const remove = async (): Promise<void> => {
    if (!confirm(t("forum.reply.delete_confirm"))) return;
    setBusy(true);
    setError(null);
    try {
      await props.onDelete(props.reply.revision);
    } catch {
      setError(t("forum.reply.delete_failed"));
    } finally {
      setBusy(false);
    }
  };

  const moderationActions = (): ReadonlyArray<ForumReplyModerationAction> =>
    props.reply.content_state === "hidden" ? ["restore"] : ["hide"];

  return (
    <ForumPost
      id={`forum-reply-${props.reply.reply_id}`}
      number={props.number}
      author={props.reply.author}
      isOriginalPoster={props.isOriginalPoster}
      createdAt={props.reply.created_at}
      editedAt={props.reply.edited_at}
      contentState={props.reply.content_state}
      actions={
        <>
          <Show when={props.onQuote && props.reply.content_state === "visible"}>
            <button class="forum-text-button" type="button" onClick={() => props.onQuote?.()}>
              {t("forum.post.quote")}
            </button>
          </Show>
          <Show when={props.own && props.reply.content_state !== "deleted"}>
            <Show when={props.reply.content_state === "visible"}>
              <button
                class="forum-text-button"
                type="button"
                disabled={busy()}
                onClick={() => setEditing(true)}
              >
                {t("forum.reply.edit")}
              </button>
            </Show>
            <button
              class="forum-text-button forum-text-button--danger"
              type="button"
              disabled={busy()}
              onClick={() => void remove()}
            >
              {t("forum.reply.delete")}
            </button>
          </Show>
          <Show when={props.canModerate && props.reply.content_state !== "deleted"}>
            <ForumModerationControls
              idPrefix={`forum-reply-${props.reply.reply_id}-moderation`}
              actions={moderationActions()}
              onModerate={(action, reason) =>
                props.onModerate(
                  action as ForumReplyModerationAction,
                  reason,
                  props.reply.revision,
                )
              }
            />
          </Show>
        </>
      }
    >
      <Show
        when={editing()}
        fallback={<ForumPostBody body={props.reply.body ?? t("forum.topic.masked")} />}
      >
        <form class="forum-form" onSubmit={(event) => void update(event)}>
          <label
            class="forum-label"
            for={`forum-reply-${props.reply.reply_id}-body`}
          >
            {t("forum.reply.edit")}
            <textarea
              id={`forum-reply-${props.reply.reply_id}-body`}
              class="forum-textarea"
              value={body()}
              onInput={(event) => setBody(event.currentTarget.value)}
              maxlength={40_000}
              required
              disabled={busy()}
            />
          </label>
          <div class="forum-actions">
            <button
              class="forum-button"
              type="button"
              disabled={busy()}
              onClick={() => setEditing(false)}
            >
              {t("common.cancel")}
            </button>
            <button
              class="forum-button forum-button--primary"
              type="submit"
              disabled={busy()}
            >
              {t("forum.reply.save")}
            </button>
          </div>
        </form>
      </Show>
      <Show when={error()}>
        {(message) => (
          <div class="forum-alert forum-alert--error" role="alert">
            {message()}
          </div>
        )}
      </Show>
    </ForumPost>
  );
}
