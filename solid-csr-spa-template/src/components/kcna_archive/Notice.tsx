import { createEffect, For, Show, type Component } from "solid-js";

import { t } from "../../state/i18n";
import { failureKey, failureKind, isNotReady, type FailureKind } from "../../utils/kcna_archive/failure";

/** A short line telling the reader what state the archive is in, with an optional retry. */
export const Notice: Component<{
  readonly kind: FailureKind;
  readonly onRetry?: () => void;
}> = (props) => (
  <p
    class={["kcna-notice", { "kcna-notice--error": !isNotReady(props.kind) }]}
    role={isNotReady(props.kind) ? "status" : "alert"}
  >
    <span class="kcna-notice-text">{t(failureKey(props.kind))}</span>
    <Show when={props.onRetry}>
      {(retry) => (
        <button type="button" class="kcna-button" onClick={() => retry()()}>
          {t("kcna.state.retry")}
        </button>
      )}
    </Show>
  </p>
);

/**
 * A failed request. A snapshot that is missing, loading, or broken is a state
 * of the server and reads as a plain notice; anything else is an error.
 */
export const Failure: Component<{ readonly error: unknown; readonly onRetry?: () => void }> = (props) => {
  const kind = (): FailureKind => failureKind(props.error);
  // The text shown is general; the console keeps the cause for the operator.
  createEffect(
    () => props.error,
    (error) => {
      if (!isNotReady(failureKind(error))) console.error(error);
    },
  );
  return <Notice kind={kind()} onRetry={props.onRetry} />;
};

const ROWS = [0, 1, 2, 3, 4, 5, 6, 7] as const;

/** Placeholder rows shown while the first results load. */
export const ListSkeleton: Component = () => (
  <div class="kcna-skeleton-list" role="status" aria-busy="true" aria-label={t("common.loading")}>
    <For each={ROWS}>
      {() => (
        <div class="kcna-skeleton-row">
          <span class="kcna-skeleton kcna-skeleton--meta" />
          <span class="kcna-skeleton kcna-skeleton--title" />
          <span class="kcna-skeleton kcna-skeleton--text" />
        </div>
      )}
    </For>
  </div>
);

/** Placeholder for a document that is loading. */
export const DocumentSkeleton: Component = () => (
  <div class="kcna-skeleton-document" role="status" aria-busy="true" aria-label={t("common.loading")}>
    <span class="kcna-skeleton kcna-skeleton--meta" />
    <span class="kcna-skeleton kcna-skeleton--heading" />
    <For each={ROWS}>{() => <span class="kcna-skeleton kcna-skeleton--text" />}</For>
  </div>
);

/** Placeholder for a chart that is loading. */
export const ChartSkeleton: Component = () => (
  <div class="kcna-skeleton kcna-skeleton--chart" role="status" aria-busy="true" aria-label={t("common.loading")} />
);
