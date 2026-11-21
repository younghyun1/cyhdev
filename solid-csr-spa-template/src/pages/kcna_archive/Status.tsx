import { createMemo, createSignal, Errored, For, Loading, refresh, Show, type Component } from "solid-js";

import { ChartSkeleton, Failure, Notice } from "../../components/kcna_archive/Notice";
import {
  kcnaArchiveApi,
  type ArchiveLanguage,
  type ArchiveSource,
  type ArchiveStatusResponse,
} from "../../services/contracts/kcna_archive";
import { locale, t } from "../../state/i18n";
import { languageKey, sourceKey } from "../../utils/kcna_archive/constants";
import { failureKind, type FailureKind } from "../../utils/kcna_archive/failure";
import { formatBytes, formatCount, formatDate, formatTimestamp } from "../../utils/kcna_archive/format";
import KcnaArchiveLayout from "./Layout";

type Reload = { readonly kind: "done" } | { readonly kind: "failed"; readonly failure: FailureKind };

/** Counts as bars, each scaled to the largest. */
const Bars = <T extends ArchiveLanguage | ArchiveSource>(props: {
  readonly values: ReadonlyArray<{ readonly value: T; readonly count: number }>;
  readonly label: (value: T) => string;
}) => {
  const most = (): number => props.values.reduce((peak, entry) => Math.max(peak, entry.count), 0);
  return (
    <ul class="kcna-bars">
      <For each={props.values}>
        {(entry) => (
          <li class="kcna-bar-row">
            <span class="kcna-bar-label" title={props.label(entry.value)}>
              {props.label(entry.value)}
            </span>
            <span class="kcna-bar-track" aria-hidden="true">
              <span
                class="kcna-bar-fill"
                style={{ transform: `scaleX(${most() === 0 ? 0 : entry.count / most()})` }}
              />
            </span>
            <span class="kcna-bar-value">{formatCount(entry.count, locale())}</span>
          </li>
        )}
      </For>
    </ul>
  );
};

const Stat: Component<{ readonly label: string; readonly value: string }> = (props) => (
  <div class="kcna-stat">
    <dt>{props.label}</dt>
    <dd>{props.value}</dd>
  </div>
);

/** The notice for a service state that cannot answer queries; a ready service has none. */
const stateNotice = (state: ArchiveStatusResponse["state"]): FailureKind | undefined => {
  switch (state) {
    case "not_installed":
      return "notInstalled";
    case "loading":
      return "loading";
    case "failed":
      return "loadFailed";
    case "ready":
      return undefined;
  }
};

const KcnaArchiveStatus: Component = () => {
  const status = createMemo(async () => kcnaArchiveApi.status());
  const [reloading, setReloading] = createSignal(false);
  const [reload, setReload] = createSignal<Reload>();

  const reloadSnapshot = async (): Promise<void> => {
    if (reloading()) return;
    setReloading(true);
    setReload(undefined);
    try {
      await kcnaArchiveApi.reload();
      setReload({ kind: "done" });
    } catch (error) {
      setReload({ kind: "failed", failure: failureKind(error) });
    } finally {
      setReloading(false);
    }
    // The reply carries the new status, but the page reads it once, like on arrival.
    void refresh(status);
  };

  return (
    <section class="kcna-status" aria-label={t("kcna.status.title")}>
      <div class="kcna-status-head">
        <h2 class="section-title kcna-section-title">{t("kcna.status.title")}</h2>
        <button
          type="button"
          class="kcna-button"
          disabled={reloading()}
          aria-busy={reloading() ? "true" : "false"}
          onClick={() => void reloadSnapshot()}
        >
          {reloading() ? t("kcna.status.reloading") : t("kcna.status.reload")}
        </button>
      </div>
      <Show when={reload()}>
        {(outcome) => {
          const current = outcome();
          return current.kind === "done" ? (
            <p class="kcna-notice kcna-notice--ok" role="status">
              <span class="kcna-notice-text">{t("kcna.status.reloaded")}</span>
            </p>
          ) : (
            <Notice kind={current.failure} />
          );
        }}
      </Show>
      <Errored
        fallback={(error, reset) => (
          <Failure
            error={error()}
            onRetry={() => {
              void refresh(status);
              reset();
            }}
          />
        )}
      >
        <Loading fallback={<ChartSkeleton />}>
          <Show when={stateNotice(status().state)}>
            {(kind) => <Notice kind={kind()} onRetry={() => void refresh(status)} />}
          </Show>
          <Show when={status().reloading}>
            <p class="kcna-notice" role="status">
              <span class="kcna-notice-text">{t("kcna.status.reloading")}</span>
            </p>
          </Show>
          <Show when={status().lastReloadFailed}>
            <Notice kind="reloadFailed" />
          </Show>
          <Show when={status().snapshot}>
            {(snapshot) => (
              <>
                <dl class="kcna-stats">
                  <Stat label={t("kcna.status.indexed")} value={formatCount(snapshot().indexed, locale())} />
                  <Stat label={t("kcna.status.documents")} value={formatCount(snapshot().documents, locale())} />
                  <Stat label={t("kcna.status.copies")} value={formatCount(snapshot().copies, locale())} />
                  <Stat
                    label={t("kcna.status.groups")}
                    value={formatCount(snapshot().translationGroups, locale())}
                  />
                  <Stat
                    label={t("kcna.status.coverage")}
                    value={`${snapshot().oldestDate ? formatDate(snapshot().oldestDate ?? "", locale()) : t("common.n_a")} – ${
                      snapshot().newestDate ? formatDate(snapshot().newestDate ?? "", locale()) : t("common.n_a")
                    }`}
                  />
                  <Stat
                    label={t("kcna.status.last_retrieved")}
                    value={
                      snapshot().lastRetrievedAt
                        ? formatTimestamp(snapshot().lastRetrievedAt ?? "", locale())
                        : t("common.n_a")
                    }
                  />
                  <Stat
                    label={t("kcna.status.snapshot_built")}
                    value={formatTimestamp(snapshot().snapshotBuiltAt, locale())}
                  />
                  <Stat
                    label={t("kcna.status.memory")}
                    value={
                      snapshot().memoryBytes === null || snapshot().memoryBytes === undefined
                        ? t("common.n_a")
                        : formatBytes(snapshot().memoryBytes ?? 0, locale())
                    }
                  />
                </dl>
                <div class="kcna-status-columns">
                  <section class="kcna-panel">
                    <h3 class="kcna-panel-title">{t("kcna.status.by_language")}</h3>
                    <Bars values={snapshot().languages} label={(value) => t(languageKey(value))} />
                  </section>
                  <section class="kcna-panel">
                    <h3 class="kcna-panel-title">{t("kcna.status.by_source")}</h3>
                    <Bars values={snapshot().sources} label={(value) => t(sourceKey(value))} />
                  </section>
                </div>
              </>
            )}
          </Show>
        </Loading>
      </Errored>
    </section>
  );
};

/** The route component: the section frame around this tab's page. */
const KcnaArchiveStatusPage: Component = () => (
  <KcnaArchiveLayout>
    <KcnaArchiveStatus />
  </KcnaArchiveLayout>
);

export default KcnaArchiveStatusPage;
