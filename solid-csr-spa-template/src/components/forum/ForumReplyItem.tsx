import { Show, createSignal, untrack } from "solid-js";

import type {
  ForumReply,
  ForumReplyModerationAction,
} from "../../services/contracts/forum_types";
import { locale, t, tx } from "../../state/i18n";
import ForumAuthorBadge from "./ForumAuthor";
import ForumModerationControls from "./ForumModerationControls";

const encoder = new TextEncoder();

interface ForumReplyItemProps {
  readonly reply: ForumReply;
  readonly own: boolean;
  readonly canModerate: boolean;
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
  const masked = () => props.reply.content_state !== "visible";
  const date = () => new Date(props.reply.created_at).toLocaleString(locale());

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
    <article
      id={`forum-reply-${props.reply.reply_id}`}
      class={["forum-reply", masked() ? "forum-reply--masked" : ""]}
    >
      <div class="forum-reply__meta">
        <ForumAuthorBadge author={props.reply.author} />
        <span>{tx("forum.topic.created", { date: date() })}</span>
        <Show when={props.reply.edited_at}>
          <span>{t("forum.topic.edited")}</span>
        </Show>
      </div>
      <Show
        when={editing()}
        fallback={
          <p class="forum-reply__body">
            {props.reply.body ?? t("forum.topic.masked")}
          </p>
        }
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
      <footer class="forum-reply__footer">
        <span class="forum-badge">{props.reply.content_state}</span>
        <Show when={props.own && props.reply.content_state !== "deleted"}>
          <div class="forum-actions">
            <Show when={props.reply.content_state === "visible"}>
              <button
                class="forum-button"
                type="button"
                disabled={busy()}
                onClick={() => setEditing(true)}
              >
                {t("forum.reply.edit")}
              </button>
            </Show>
            <button
              class="forum-button forum-button--danger"
              type="button"
              disabled={busy()}
              onClick={() => void remove()}
            >
              {t("forum.reply.delete")}
            </button>
          </div>
        </Show>
      </footer>
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
    </article>
  );
}
