import { pageStyles } from "../styles/pageStyles";
import { t } from "../state/i18n";
import { Show } from "solid-js";
import { useLocation } from "@solidjs/router";

export default function NotFound() {
  const location = useLocation();
  const construction = () => location.pathname === "/under-construction";
  return (
    <main class={pageStyles.page}>
      <div
        class={`${pageStyles.pageInnerNarrow} flex items-center justify-center min-h-[70vh]`}
      >
        <div class={`${pageStyles.cardPadded} w-full`}>
          <div class="flex items-start gap-4">
            <div class="flex h-12 w-12 shrink-0 items-center justify-center rounded-sm bg-accent-soft text-accent ring-1 ring-accent/30">
              <span class="text-xl" aria-hidden="true">
                {construction() ? "🚧" : "404"}
              </span>
            </div>

            <div class="min-w-0">
              <h1 class={`${pageStyles.titleSm} mb-1`}>
                {t(construction() ? "not_found.title" : "page.not_found.title")}
              </h1>
              <p class={pageStyles.muted}>{t(construction() ? "not_found.message" : "not_found.missing_message")}</p>
            </div>
          </div>

          <Show when={construction()}>
          <div class="mt-6 grid gap-4 sm:grid-cols-3">
            <div class={`${pageStyles.card} p-4`}>
              <p class="text-sm font-semibold">{t("not_found.status_label")}</p>
              <p class={`mt-1 ${pageStyles.muted}`}>
                {t("not_found.status_value")}
              </p>
            </div>
            <div class={`${pageStyles.card} p-4`}>
              <p class="text-sm font-semibold">{t("not_found.eta_label")}</p>
              <p class={`mt-1 ${pageStyles.muted}`}>
                {t("not_found.eta_value")}
              </p>
            </div>
            <div class={`${pageStyles.card} p-4`}>
              <p class="text-sm font-semibold">
                {t("not_found.meanwhile_label")}
              </p>
              <p class={`mt-1 ${pageStyles.muted}`}>
                {t("not_found.meanwhile_value")}
              </p>
            </div>
          </div>
          </Show>
          <div class="mt-6 flex flex-wrap gap-3">
            <a href="/" class={pageStyles.buttonPrimary}>
              {t("common.go_home")}
            </a>
            <button
              type="button"
              onClick={() => history.back()}
              class={pageStyles.buttonSecondary}
            >
              {t("common.go_back")}
            </button>
          </div>
        </div>
      </div>
    </main>
  );
}
