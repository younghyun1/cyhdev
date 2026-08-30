import { For, Show, createEffect, createSignal, untrack } from "solid-js";

import type {
  ForumReplyModerationAction,
  ForumTopicModerationAction,
} from "../../services/contracts/forum_types";
import { t } from "../../state/i18n";

const encoder = new TextEncoder();

type ModerationAction =
  | ForumTopicModerationAction
  | ForumReplyModerationAction;

interface ForumModerationControlsProps {
  readonly idPrefix: string;
  readonly actions: ReadonlyArray<ModerationAction>;
  readonly onModerate: (
    action: ModerationAction,
    reason: string,
  ) => Promise<void>;
}

const actionLabel = (action: ModerationAction): string => {
  switch (action) {
    case "hide":
      return t("forum.moderation.hide");
    case "restore":
      return t("forum.moderation.restore");
    case "lock":
      return t("forum.moderation.lock");
    case "unlock":
      return t("forum.moderation.unlock");
    case "pin":
      return t("forum.moderation.pin");
    case "unpin":
      return t("forum.moderation.unpin");
    default:
      return action;
  }
};

export default function ForumModerationControls(
  props: ForumModerationControlsProps,
) {
  const [action, setAction] = createSignal<ModerationAction | null>(
    untrack(() => props.actions[0] ?? null),
  );
  const [reason, setReason] = createSignal("");
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);

  createEffect(
    () => props.actions,
    (actions) => {
      const current = action();
      if (current === null || !actions.includes(current)) {
        setAction(actions[0] ?? null);
      }
    },
  );

  const submit = async (event: SubmitEvent): Promise<void> => {
    event.preventDefault();
    const selected = action();
    const nextReason = reason().trim();
    const reasonCharacters = Array.from(nextReason).length;
    if (
      !selected ||
      reasonCharacters < 8 ||
      reasonCharacters > 500 ||
      encoder.encode(nextReason).byteLength > 2_000
    ) {
      setError(t("forum.moderation.reason_help"));
      return;
    }
    setBusy(true);
    setError(null);
    try {
      await props.onModerate(selected, nextReason);
      setReason("");
    } catch {
      setError(t("forum.moderation.failed"));
    } finally {
      setBusy(false);
    }
  };

  return (
    <form
      class="forum-panel forum-moderation"
      onSubmit={(event) => void submit(event)}
    >
      <h3 class="forum-section-title">{t("forum.moderation.title")}</h3>
      <div class="forum-moderation__controls">
        <label class="forum-label" for={`${props.idPrefix}-action`}>
          <span class="forum-help">{t("forum.moderation.title")}</span>
          <select
            id={`${props.idPrefix}-action`}
            class="forum-select"
            value={action() ?? ""}
            onChange={(event) =>
              setAction(event.currentTarget.value as ModerationAction)
            }
            disabled={busy() || props.actions.length === 0}
          >
            <For each={props.actions}>
              {(item) => <option value={item}>{actionLabel(item)}</option>}
            </For>
          </select>
        </label>
        <button
          class="forum-button forum-button--primary"
          type="submit"
          disabled={busy() || props.actions.length === 0}
        >
          {busy()
            ? t("forum.moderation.applying")
            : t("forum.moderation.apply")}
        </button>
      </div>
      <label class="forum-label" for={`${props.idPrefix}-reason`}>
        {t("forum.moderation.reason")}
        <textarea
          id={`${props.idPrefix}-reason`}
          class="forum-textarea forum-textarea--compact"
          value={reason()}
          onInput={(event) => setReason(event.currentTarget.value)}
          minlength={8}
          maxlength={1_000}
          required
          disabled={busy()}
        />
        <span class="forum-help">{t("forum.moderation.reason_help")}</span>
      </label>
      <Show when={error()}>
        {(message) => (
          <div class="forum-alert forum-alert--error" role="alert">
            {message()}
          </div>
        )}
      </Show>
    </form>
  );
}
