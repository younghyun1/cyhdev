import { Show, createSignal, untrack } from "solid-js";

import { t } from "../../state/i18n";

interface ForumTopicFormProps {
  readonly idPrefix: string;
  readonly initialTitle?: string;
  readonly initialBody?: string;
  readonly busy: boolean;
  readonly mode: "create" | "update";
  readonly onSubmit: (title: string, body: string) => Promise<void>;
  readonly onCancel?: () => void;
}

const encoder = new TextEncoder();
const characterCount = (value: string) => Array.from(value).length;

export default function ForumTopicForm(props: ForumTopicFormProps) {
  const initial = untrack(() => ({
    title: props.initialTitle ?? "",
    body: props.initialBody ?? "",
  }));
  const [title, setTitle] = createSignal(initial.title);
  const [body, setBody] = createSignal(initial.body);
  const [error, setError] = createSignal<string | null>(null);

  const submit = async (event: SubmitEvent): Promise<void> => {
    event.preventDefault();
    const nextTitle = title().trim();
    const nextBody = body().trim();
    if (!nextTitle || !nextBody) {
      setError(t("forum.form.required"));
      return;
    }
    if (
      characterCount(nextTitle) < 3 ||
      characterCount(nextTitle) > 160 ||
      encoder.encode(nextTitle).byteLength > 512
    ) {
      setError(t("forum.form.title_invalid"));
      return;
    }
    if (
      characterCount(nextBody) > 20_000 ||
      encoder.encode(nextBody).byteLength > 65_536
    ) {
      setError(t("forum.form.body_invalid"));
      return;
    }
    setError(null);
    await props.onSubmit(nextTitle, nextBody);
  };

  return (
    <form class="forum-form" onSubmit={(event) => void submit(event)}>
      <label class="forum-label" for={`${props.idPrefix}-title`}>
        {t("forum.form.title_label")}
        <input
          id={`${props.idPrefix}-title`}
          class="forum-input"
          value={title()}
          onInput={(event) => setTitle(event.currentTarget.value)}
          minlength={3}
          maxlength={320}
          required
          disabled={props.busy}
        />
        <span class="forum-help">{t("forum.form.title_help")}</span>
      </label>
      <label class="forum-label" for={`${props.idPrefix}-body`}>
        {t("forum.form.body_label")}
        <textarea
          id={`${props.idPrefix}-body`}
          class="forum-textarea"
          value={body()}
          onInput={(event) => setBody(event.currentTarget.value)}
          maxlength={40_000}
          required
          disabled={props.busy}
        />
        <span class="forum-help">{t("forum.form.body_help")}</span>
      </label>
      <Show when={error()}>
        {(message) => (
          <div class="forum-alert forum-alert--error" role="alert">
            {message()}
          </div>
        )}
      </Show>
      <div class="forum-actions">
        <Show when={props.onCancel}>
          <button
            class="forum-button"
            type="button"
            disabled={props.busy}
            onClick={() => props.onCancel?.()}
          >
            {t("common.cancel")}
          </button>
        </Show>
        <button
          class="forum-button forum-button--primary"
          type="submit"
          disabled={props.busy}
        >
          {props.busy
            ? props.mode === "create"
              ? t("forum.form.creating")
              : t("forum.form.updating")
            : props.mode === "create"
              ? t("forum.form.create")
              : t("forum.form.update")}
        </button>
      </div>
    </form>
  );
}
