import { Show, createMemo, createSignal } from "solid-js";

import { t } from "../../state/i18n";

const encoder = new TextEncoder();
const MAX_REPLY_CHARACTERS = 20_000;
export const REPLY_COMPOSER_ID = "forum-new-reply";

interface ForumReplyComposerProps {
  readonly value: string;
  readonly onInput: (value: string) => void;
  readonly onSubmit: (body: string) => Promise<void>;
}

/** Reply box under a thread; its text lives in the page so posts can quote into it. */
export default function ForumReplyComposer(props: ForumReplyComposerProps) {
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const count = createMemo(() => Array.from(props.value).length);

  const submit = async (): Promise<void> => {
    if (busy()) return;
    const next = props.value.trim();
    if (
      !next ||
      Array.from(next).length > MAX_REPLY_CHARACTERS ||
      encoder.encode(next).byteLength > 65_536
    ) {
      setError(t("forum.form.body_invalid"));
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await props.onSubmit(next);
      props.onInput("");
    } catch {
      setError(t("forum.reply.create_failed"));
    } finally {
      setBusy(false);
    }
  };

  return (
    <form
      class="forum-form forum-composer"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <label class="forum-label" for={REPLY_COMPOSER_ID}>
        {t("forum.topic.reply_action")}
      </label>
      <textarea
        id={REPLY_COMPOSER_ID}
        class="forum-textarea"
        value={props.value}
        placeholder={t("forum.reply.placeholder")}
        onInput={(event) => props.onInput(event.currentTarget.value)}
        onKeyDown={(event) => {
          if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
            event.preventDefault();
            void submit();
          }
        }}
        maxlength={40_000}
        required
        disabled={busy()}
      />
      <Show when={error()}>
        {(message) => (
          <div class="forum-alert forum-alert--error" role="alert">
            {message()}
          </div>
        )}
      </Show>
      <div class="forum-composer__footer">
        <p class="forum-help">
          {t("forum.reply.shortcut")}{" "}
          <span class="forum-composer__count">
            {count()}/{MAX_REPLY_CHARACTERS}
          </span>
        </p>
        <button class="forum-button forum-button--primary" type="submit" disabled={busy()}>
          {busy() ? t("forum.reply.submitting") : t("forum.reply.submit")}
        </button>
      </div>
    </form>
  );
}
