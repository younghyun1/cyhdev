import { Show, createSignal } from "solid-js";

import type { AdminOperationsApi } from "../../../services/contracts/admin_operations";
import { adminOperationsApi } from "../../../services/contracts/admin_operations";
import { loadUiTextBundle, locale, t, tx } from "../../../state/i18n";
import { operationErrorMessage } from "./operationsFormat";

type Props = {
  readonly service?: Pick<AdminOperationsApi, "syncI18n">;
};

export default function I18nSyncPanel(props: Props) {
  const service = () => props.service ?? adminOperationsApi;
  const [busy, setBusy] = createSignal(false);
  const [error, setError] = createSignal<string | null>(null);
  const [loadedRows, setLoadedRows] = createSignal<number | null>(null);

  const synchronize = async () => {
    if (busy()) return;
    setBusy(true);
    setError(null);
    setLoadedRows(null);
    try {
      const response = await service().syncI18n();
      setLoadedRows(response.data.num_rows);
      await loadUiTextBundle(locale());
    } catch (syncError: unknown) {
      setError(operationErrorMessage(syncError));
    } finally {
      setBusy(false);
    }
  };

  return (
    <section class="operations-panel" aria-labelledby="i18n-heading">
      <div class="operations-panel-heading">
        <div>
          <h2 id="i18n-heading">{t("operations.i18n.title")}</h2>
          <p>{t("operations.i18n.description")}</p>
        </div>
        <button
          type="button"
          class="operations-primary-button"
          disabled={busy()}
          onClick={() => void synchronize()}
        >
          {busy()
            ? t("operations.i18n.synchronizing")
            : t("operations.i18n.synchronize")}
        </button>
      </div>
      <Show when={error() !== null}>
        <div class="operations-error" role="alert">
          <p>{error()}</p>
          <p>{t("operations.i18n.incomplete_error")}</p>
        </div>
      </Show>
      <Show when={loadedRows() !== null}>
        <p class="operations-receipt" role="status">
          {tx("operations.i18n.success", { count: loadedRows() ?? 0 })}
        </p>
      </Show>
    </section>
  );
}
