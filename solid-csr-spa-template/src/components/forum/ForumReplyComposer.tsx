import { Show, createSignal } from "solid-js";

import { t } from "../../state/i18n";

const encoder = new TextEncoder();

interface ForumReplyComposerProps {
  readonly disabled: boolean;
  readonly onSubmit: (body: string) => Promise<void>;
}

export default function ForumReplyComposer(props: ForumReplyComposerProps) {
  const [body, setBody] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  const submit = async (event: SubmitEvent): Promise<void> => {
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
      await props.onSubmit(next);
      setBody("");
    } catch {
      setError(t("forum.reply.create_failed"));
    } finally {
      setBusy(false);
    }
  };

  return (
    <form class="forum-form" onSubmit={(event) => void submit(event)}>
      <label class="forum-label" for="forum-new-reply">
        {t("forum.reply.heading")}
        <textarea
          id="forum-new-reply"
          class="forum-textarea"
          value={body()}
          placeholder={t("forum.reply.placeholder")}
          onInput={(event) => setBody(event.currentTarget.value)}
          maxlength={40_000}
          required
          disabled={busy() || props.disabled}
        />
      </label>
      <Show when={props.disabled}>
        <p class="forum-alert">{t("forum.reply.locked")}</p>
      </Show>
      <Show when={error()}>
        {(message) => (
          <div class="forum-alert forum-alert--error" role="alert">
            {message()}
          </div>
        )}
      </Show>
      <button
        class="forum-button forum-button--primary"
        type="submit"
        disabled={busy() || props.disabled}
      >
        {busy() ? t("forum.reply.submitting") : t("forum.reply.submit")}
      </button>
    </form>
  );
}
