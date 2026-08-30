// Processing popup: live per-batch / per-item status for the current user.
//
// Controlled by the parent (photographs page) via `show`/`onClose`. Reads the
// module-scoped batch store directly. Reuses the page's `.modal-overlay` /
// `.modal-content` shell plus a `.processing-modal` rule in the inline <style>.

import { Show, createMemo, onSettled } from "solid-js";
import { Key } from "@solid-primitives/keyed";
import {
  batches,
  clearFinishedBatches,
  type BatchEntry,
} from "../../state/photo_batches";
import { pageStyles, chipClass } from "../../styles/pageStyles";
import { t, tx } from "../../state/i18n";
import type { UiTextKey } from "../../i18n/keys";
import type { ProcessingStatus } from "../../generated";

interface ProcessingModalProps {
  show: boolean;
  onClose: () => void;
}

const STATUS_LABEL: Record<ProcessingStatus["status"], UiTextKey> = {
  queued: "photos.status_queued",
  encoding: "photos.status_encoding",
  uploading: "photos.status_uploading",
  persisting: "photos.status_persisting",
  completed: "photos.status_completed",
  failed: "photos.status_failed",
};

export default function ProcessingModal(props: ProcessingModalProps) {
  const visibleBatches = createMemo<BatchEntry[]>(() =>
    Object.keys(batches)
      .map((id) => batches[id])
      .filter((entry): entry is BatchEntry => entry !== undefined && !entry._missing)
      .sort((a, b) => {
        const at = a.status?.created_at ?? "";
        const bt = b.status?.created_at ?? "";
        return bt.localeCompare(at);
      }),
  );

  const onKeyDown = (e: KeyboardEvent) => {
    if (e.key === "Escape") props.onClose();
  };

  onSettled(() => {
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  });

  const progressPercent = (entry: BatchEntry): number => {
    const status = entry.status;
    if (!status || status.total === 0) return 0;
    const finished = status.completed + status.failed;
    return Math.min(100, Math.max(0, Math.round((finished / status.total) * 100)));
  };

  return (
    <Show when={props.show}>
      <div
        class="modal-overlay"
        onClick={(e) => {
          if (e.target === e.currentTarget) props.onClose();
        }}
      >
        <div
          class="modal-content processing-modal p-6"
          role="dialog"
          aria-modal="true"
          aria-label={t("photos.processing")}
        >
          <div class="flex items-center justify-between mb-4">
            <h2 class={pageStyles.sectionTitle}>{t("photos.processing")}</h2>
            <div class="flex items-center gap-2">
              <button
                type="button"
                class={pageStyles.buttonSecondary}
                onClick={() => clearFinishedBatches()}
              >
                {t("photos.batch_clear_finished")}
              </button>
              <button
                type="button"
                class={pageStyles.buttonGhost}
                onClick={() => props.onClose()}
                aria-label={t("common.close")}
              >
                ✕
              </button>
            </div>
          </div>

          <Show
            when={visibleBatches().length > 0}
            fallback={<p class={pageStyles.muted}>{t("photos.batch_no_active")}</p>}
          >
            <div class="flex flex-col gap-5">
              <Key each={visibleBatches()} by={(entry) => entry.batch_id}>
                {(entry) => (
                  <div class={`${pageStyles.card} p-4`}>
                    <p class="text-sm font-semibold mb-2" aria-live="polite">
                      {tx("photos.batch_summary", {
                        completed: entry().status?.completed ?? 0,
                        total: entry().status?.total ?? 0,
                        failed: entry().status?.failed ?? 0,
                      })}
                    </p>
                    <div class="w-full bg-surface-2 rounded-sm h-2 overflow-hidden mb-3">
                      <div
                        class="bg-ink h-2 transition-all duration-200"
                        style={{ width: `${progressPercent(entry())}%` }}
                      />
                    </div>
                    <div class="flex flex-wrap gap-2">
                      <Key
                        each={entry().status?.items ?? []}
                        by={(item) => item.item_id}
                      >
                        {(item) => (
                          <span
                            class={chipClass(item().status.status)}
                            title={
                              item().status.status === "failed"
                                ? item().status.reason
                                : (item().file_name ?? undefined)
                            }
                          >
                            {(item().file_name ?? item().item_id.slice(0, 8)) +
                              " · " +
                              t(STATUS_LABEL[item().status.status])}
                          </span>
                        )}
                      </Key>
                    </div>
                  </div>
                )}
              </Key>
            </div>
          </Show>
        </div>
      </div>
    </Show>
  );
}
