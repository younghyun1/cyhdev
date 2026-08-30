import { For, Show, createMemo, createSignal } from "solid-js";
import type {
  DeleteProfilePictureResponse,
  ProfilePictureHistoryItem,
} from "../../generated";
import { t, tx } from "../../state/i18n";
import { pageStyles } from "../../styles/pageStyles";

const MAX_VISIBLE_HISTORY = 8;

type ProfilePictureHistoryProps = {
  readonly items: ReadonlyArray<ProfilePictureHistoryItem>;
  readonly maximum: number;
  readonly loading: boolean;
  readonly loadError: string | null;
  readonly onSelect: (profilePictureId: string) => Promise<void>;
  readonly onDelete: (
    profilePictureId: string,
  ) => Promise<DeleteProfilePictureResponse>;
};

function formatCreatedAt(value: string): string {
  const createdAt = new Date(value);
  return Number.isNaN(createdAt.getTime())
    ? value
    : createdAt.toLocaleString();
}

export default function ProfilePictureHistory(
  props: ProfilePictureHistoryProps,
) {
  const [pendingId, setPendingId] = createSignal<string | null>(null);
  const [error, setError] = createSignal<string | null>(null);
  const [status, setStatus] = createSignal<string | null>(null);
  const visibleItems = createMemo(() =>
    props.items.slice(0, Math.min(MAX_VISIBLE_HISTORY, props.maximum)),
  );

  const selectPicture = async (item: ProfilePictureHistoryItem) => {
    if (item.is_active || pendingId() !== null) return;
    setPendingId(item.profile_picture_id);
    setError(null);
    setStatus(null);
    try {
      await props.onSelect(item.profile_picture_id);
      setStatus(t("profile.picture_history.selected"));
    } catch (caught: unknown) {
      setError(
        caught instanceof Error
          ? caught.message
          : t("profile.picture_history.action_failed"),
      );
    } finally {
      setPendingId(null);
    }
  };

  const deletePicture = async (item: ProfilePictureHistoryItem) => {
    if (pendingId() !== null) return;
    const confirmed = window.confirm(
      tx("profile.picture_history.delete_confirmation", {
        date: formatCreatedAt(item.created_at),
      }),
    );
    if (!confirmed) return;
    setPendingId(item.profile_picture_id);
    setError(null);
    setStatus(null);
    try {
      const response = await props.onDelete(item.profile_picture_id);
      setStatus(
        response.cleanup_remaining_count > 0
          ? tx("profile.picture_history.cleanup_pending", {
              count: response.cleanup_remaining_count,
            })
          : t("profile.picture_history.deleted"),
      );
    } catch (caught: unknown) {
      setError(
        caught instanceof Error
          ? caught.message
          : t("profile.picture_history.action_failed"),
      );
    } finally {
      setPendingId(null);
    }
  };

  return (
    <div class="mt-6 border-t border-line pt-5">
      <div class="flex items-baseline justify-between gap-3">
        <h3 class="font-semibold">{t("profile.picture_history.title")}</h3>
        <span class={pageStyles.muted}>
          {tx("profile.picture_history.count", {
            count: visibleItems().length,
            maximum: Math.min(MAX_VISIBLE_HISTORY, props.maximum),
          })}
        </span>
      </div>
      <Show when={props.loadError !== null}>
        <div class={`${pageStyles.alertError} mt-3`} role="alert">
          {props.loadError}
        </div>
      </Show>
      <Show when={error()}>
        <div class={`${pageStyles.alertError} mt-3`} role="alert">
          {error()}
        </div>
      </Show>
      <Show when={status()}>
        <div class={`${pageStyles.alertSuccess} mt-3`} aria-live="polite">
          {status()}
        </div>
      </Show>
      <Show when={!props.loading} fallback={<p class={`${pageStyles.muted} mt-3`}>{t("common.loading")}</p>}>
        <Show
          when={visibleItems().length > 0}
          fallback={<p class={`${pageStyles.muted} mt-3`}>{t("profile.picture_history.empty")}</p>}
        >
          <div class="mt-3 grid grid-cols-2 gap-3 sm:grid-cols-4">
            <For each={visibleItems()}>
              {(item) => (
                <article class="overflow-hidden rounded-sm border border-line bg-surface-2">
                  <img
                    src={item.object_url || "/default-profile.png"}
                    alt={t("profile.picture_alt")}
                    class="aspect-square w-full object-cover"
                    loading="lazy"
                  />
                  <div class="space-y-2 p-2">
                    <div class="flex min-h-5 items-center justify-between gap-1">
                      <span class="truncate text-xs text-ink-muted">
                        {formatCreatedAt(item.created_at)}
                      </span>
                      <Show when={item.is_active}>
                        <span class={pageStyles.badge}>
                          {t("profile.picture_history.active")}
                        </span>
                      </Show>
                    </div>
                    <div class="flex flex-wrap gap-1">
                      <button
                        type="button"
                        class={pageStyles.buttonSecondary}
                        disabled={item.is_active || pendingId() !== null}
                        onClick={() => selectPicture(item)}
                      >
                        {t("profile.picture_history.select")}
                      </button>
                      <button
                        type="button"
                        class={pageStyles.buttonDanger}
                        disabled={pendingId() !== null}
                        onClick={() => deletePicture(item)}
                      >
                        {t("common.delete")}
                      </button>
                    </div>
                  </div>
                </article>
              )}
            </For>
          </div>
        </Show>
      </Show>
    </div>
  );
}
