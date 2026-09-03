import type { Component } from "solid-js";
import {
  Show,
  createEffect,
  createMemo,
  createSignal,
  onSettled,
} from "solid-js";
import { useLocation } from "@solidjs/router";
import {
  healthState,
  clientNow,
  setClientNow,
  refreshHealthState,
  formatIsoAge,
} from "../state/health";
import { serverBuildInfo } from "../state/server_info";
import { t } from "../state/i18n";
import { createMediaQuery } from "../utils/mediaQuery";
import { MobileDialog } from "./MobileDialog";
import SystemStatusDetails, {
  createLiveUptime,
} from "./SystemStatusDetails";

declare const __BUILD_TIMESTAMP__: string;
declare const __SOLID_VERSION__: string;

function isKeyboardControl(target: EventTarget | null): boolean {
  if (!(target instanceof HTMLElement)) return false;
  if (target.isContentEditable) return true;
  if (
    target instanceof HTMLTextAreaElement ||
    target instanceof HTMLSelectElement
  ) {
    return true;
  }
  if (!(target instanceof HTMLInputElement)) return false;
  return ![
    "button",
    "checkbox",
    "color",
    "file",
    "hidden",
    "image",
    "radio",
    "range",
    "reset",
    "submit",
  ].includes(target.type);
}

const BottomBar: Component = () => {
  const location = useLocation();
  const isMobile = createMediaQuery("(max-width: 767px)");
  const [detailsOpen, setDetailsOpen] = createSignal(false);
  const [keyboardControlFocused, setKeyboardControlFocused] =
    createSignal(false);
  const liveUptime = createLiveUptime();

  createEffect(
    () => location.pathname,
    () => {
      setDetailsOpen(false);
      void refreshHealthState();
    },
  );

  onSettled(() => {
    const interval = setInterval(() => setClientNow(new Date()), 1000);
    const updateFocus = () =>
      queueMicrotask(() =>
        setKeyboardControlFocused(
          isMobile() && isKeyboardControl(document.activeElement),
        ),
      );
    document.addEventListener("focusin", updateFocus);
    document.addEventListener("focusout", updateFocus);
    return () => {
      clearInterval(interval);
      document.removeEventListener("focusin", updateFocus);
      document.removeEventListener("focusout", updateFocus);
    };
  });

  createEffect(
    () => isMobile(),
    (mobile) => {
      if (!mobile) setDetailsOpen(false);
    },
  );

  const mobileSummary = createMemo(() => {
    const health = healthState();
    if (!health) return `${t("bottom_bar.site_status")}: …`;
    return `${t("bottom_bar.up")} ${liveUptime()} · ${health.responses_handled} ${t("bottom_bar.responses")} · ${health.users_logged_in} ${t("bottom_bar.sessions")}`;
  });

  return (
    <>
      <Show when={detailsOpen() && isMobile()}>
        <MobileDialog
          onClose={() => setDetailsOpen(false)}
          overlayClass="mobile-sheet-overlay"
          panelClass="mobile-sheet"
          ariaLabelledBy="mobile-status-title"
          initialFocusSelector="[data-status-close]"
        >
          <div class="flex items-center justify-between gap-3">
            <div
              id="mobile-status-title"
              class="text-sm font-semibold text-ink"
            >
              {t("bottom_bar.site_status")}
            </div>
            <button
              data-status-close
              type="button"
              class="ui-button px-3 py-1.5 text-xs rounded-sm border border-line text-ink hover:bg-surface-2"
              onClick={() => setDetailsOpen(false)}
              aria-label={t("common.close")}
            >
              {t("common.close")}
            </button>
          </div>
          <div class="mt-3">
            <SystemStatusDetails />
          </div>
        </MobileDialog>
      </Show>

      <footer
        data-site-bar="bottom"
        class={{
          "site-status-bar fixed bottom-0 left-0 w-full transition-colors duration-90 border-t border-line bg-paper/90 backdrop-blur text-[9px] sm:text-[11px]": true,
          "site-status-bar--keyboard-hidden": keyboardControlFocused(),
        }}
        style={{ "z-index": 50 }}
        onClick={() => {
          if (isMobile()) setDetailsOpen(true);
        }}
        role={isMobile() ? "button" : undefined}
        aria-label={isMobile() ? t("bottom_bar.open_details") : undefined}
        aria-hidden={keyboardControlFocused() ? "true" : undefined}
        tabindex={isMobile() && !keyboardControlFocused() ? 0 : undefined}
        onKeyDown={(event) => {
          if (!isMobile()) return;
          if (event.key === "Enter" || event.key === " ") {
            event.preventDefault();
            setDetailsOpen(true);
          }
          if (event.key === "Escape") setDetailsOpen(false);
        }}
      >
        <div class="w-full px-2 sm:px-3 py-0.5 sm:py-1.5 flex flex-row justify-between items-start gap-2 sm:gap-3 font-mono tabular-nums">
          <div class="hidden sm:block text-ink-muted leading-tight space-y-0.5 max-w-[55%]">
            <div>
              {t("bottom_bar.fe")}: {t("bottom_bar.built")}{" "}
              {__BUILD_TIMESTAMP__} {t("bottom_bar.with_solid")}{" "}
              {__SOLID_VERSION__}
            </div>
            <div>
              {t("bottom_bar.be")}: {t("bottom_bar.built")}{" "}
              {serverBuildInfo().built_time ?? "…"} (
              {serverBuildInfo().name ?? "…"})
              {serverBuildInfo().rust_version && (
                <> rust/{serverBuildInfo().rust_version}</>
              )}
            </div>
          </div>

          <div class="hidden sm:block text-ink-muted leading-tight text-right space-y-0.5 max-w-[45%]">
            {healthState() ? (
              (() => {
                const health = healthState()!;
                return (
                  <>
                    <div>
                      {t("bottom_bar.up")} {liveUptime()} ·{" "}
                      {t("bottom_bar.handled")} {health.responses_handled}{" "}
                      {t("bottom_bar.responses")} · {t("bottom_bar.sessions")}{" "}
                      {health.users_logged_in}
                    </div>
                    <div class="hidden xs:block sm:block">
                      {t("bottom_bar.db")} {health.db_version} ·{" "}
                      {t("bottom_bar.db_latency")} {health.db_latency}
                    </div>
                    <div class="hidden sm:block">
                      {t("bottom_bar.time_to_report")}: {health.time_to_process ?? "?"}{" "}
                      · {t("bottom_bar.net")}{" "}
                      {health.client_latency_ms?.toFixed(1) ?? "?"}ms ·{" "}
                      {t("bottom_bar.state_age")}{" "}
                      {formatIsoAge(health.timestamp, clientNow())}
                    </div>
                  </>
                );
              })()
            ) : (
              <div>{t("bottom_bar.metrics")}: …</div>
            )}
          </div>

          <div class="site-status-mobile sm:hidden w-full flex items-center justify-between gap-2 text-ink-muted leading-tight">
            <div class="truncate">{mobileSummary()}</div>
            <div class="shrink-0 text-[10px] opacity-70">
              {t("bottom_bar.tap")}
            </div>
          </div>
        </div>
      </footer>
    </>
  );
};

export default BottomBar;
