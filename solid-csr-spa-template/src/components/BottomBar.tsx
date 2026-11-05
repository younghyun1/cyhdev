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
  refreshHealthStateIfStale,
  formatIsoAge,
} from "../state/health";
import BuildDetails, { Ltr } from "./BuildDetails";
import { postgresVersion } from "../utils/statusMetadata";
import { t } from "../state/i18n";
import { createMediaQuery } from "../utils/mediaQuery";
import { MobileDialog } from "./MobileDialog";
import SystemStatusDetails, {
  createLiveUptime,
} from "./SystemStatusDetails";


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
  const copyrightYear = new Date().getUTCFullYear();
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
      void refreshHealthStateIfStale();
    },
  );

  onSettled(() => {
    const interval = setInterval(() => setClientNow(new Date()), 1000);
    let pointerActive = false;
    let focusTimer: ReturnType<typeof setTimeout> | undefined;
    const updateFocus = () =>
      queueMicrotask(() => {
        const focused = isMobile() && isKeyboardControl(document.activeElement);
        // A footer restored between pointerdown and click can cover the intended submit button.
        if (focused || !pointerActive) setKeyboardControlFocused(focused);
      });
    const startPointer = () => { pointerActive = true; };
    const finishPointer = () => {
      pointerActive = false;
      clearTimeout(focusTimer);
      focusTimer = setTimeout(updateFocus, 0);
    };
    // Returning to the tab or window is when stale counters are most visible.
    const refreshOnReturn = () => {
      if (!document.hidden) void refreshHealthStateIfStale();
    };
    document.addEventListener("focusin", updateFocus);
    document.addEventListener("focusout", updateFocus);
    document.addEventListener("pointerdown", startPointer, true);
    document.addEventListener("pointerup", finishPointer, true);
    document.addEventListener("pointercancel", finishPointer, true);
    document.addEventListener("visibilitychange", refreshOnReturn);
    window.addEventListener("focus", refreshOnReturn);
    return () => {
      clearInterval(interval);
      clearTimeout(focusTimer);
      document.removeEventListener("focusin", updateFocus);
      document.removeEventListener("focusout", updateFocus);
      document.removeEventListener("pointerdown", startPointer, true);
      document.removeEventListener("pointerup", finishPointer, true);
      document.removeEventListener("pointercancel", finishPointer, true);
      document.removeEventListener("visibilitychange", refreshOnReturn);
      window.removeEventListener("focus", refreshOnReturn);
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
    // Each value is a left-to-right isolate (LRI ... PDI) so digits and units keep their
    // order beside translated labels in a right-to-left sentence.
    const value = (text: string | number) => `\u2066${text}\u2069`;
    return `${t("bottom_bar.up")} ${value(liveUptime())} · ${value(health.responses_handled)} ${t("bottom_bar.responses")} · ${value(health.users_logged_in)} ${t("bottom_bar.sessions")}`;
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
          "site-status-bar fixed inset-x-0 bottom-0 transition-colors duration-90 border-t border-line bg-paper/90 backdrop-blur text-[9px] sm:text-[11px]": true,
          "site-status-bar--keyboard-hidden": keyboardControlFocused(),
        }}
        style={{ "z-index": 50 }}
        aria-hidden={keyboardControlFocused() ? "true" : undefined}
      >
        <div
          class="site-status-layout font-mono tabular-nums"
          // Status lines are cut to keep the bar one height; hovering reveals the full text.
          onPointerOver={(event) => {
            if (event.target instanceof HTMLElement) event.target.title = event.target.textContent ?? "";
          }}
        >
          <div class="site-status-builds text-ink-muted leading-tight space-y-0.5">
            <BuildDetails />
          </div>

          <div class="site-status-metrics text-ink-muted leading-tight text-end space-y-0.5">
            {healthState() ? (
              (() => {
                const health = healthState()!;
                return (
                  <>
                    <div>
                      {t("bottom_bar.up")} <Ltr>{liveUptime()}</Ltr> ·{" "}
                      {t("bottom_bar.handled")} <Ltr>{health.responses_handled}</Ltr>{" "}
                      {t("bottom_bar.responses")} · {t("bottom_bar.sessions")}{" "}
                      <Ltr>{health.users_logged_in}</Ltr>
                    </div>
                    <div>
                      <Ltr>{postgresVersion(health.db_version)}</Ltr> ·{" "}
                      {t("bottom_bar.db_latency")} <Ltr>{health.db_latency}</Ltr>
                    </div>
                  </>
                );
              })()
            ) : (
              <div>{t("bottom_bar.metrics")}: …</div>
            )}
          </div>

          <div class="site-status-timing text-ink-muted leading-tight text-end">
            <Show when={healthState()}>
              {(health) => <>
                {t("bottom_bar.time_to_report")}: <Ltr>{health().time_to_process ?? "?"}</Ltr>{" "}
                · {t("bottom_bar.net")} <Ltr>{health().client_latency_ms?.toFixed(1) ?? "?"}ms</Ltr> ·{" "}
                {t("bottom_bar.state_age")} <Ltr>{formatIsoAge(health().timestamp, clientNow())}</Ltr>
              </>}
            </Show>
          </div>
          <button
            type="button"
            class="site-status-mobile w-full flex items-center justify-between gap-2 text-ink-muted leading-tight"
            aria-description={t("bottom_bar.open_details")}
            onClick={() => setDetailsOpen(true)}
          >
            <span class="truncate">{mobileSummary()}</span>
            <span class="shrink-0 text-[10px] opacity-70">
              {t("bottom_bar.tap")}
            </span>
          </button>
          <div class="site-copyright">
            <span dir="ltr">© 2025-{copyrightYear} Young Hyun Chi.</span>
            <span dir="ltr"><a href="https://github.com/younghyun1/cyhdev" target="_blank" rel="noopener noreferrer">Code</a> licensed under the MIT License.</span>
          </div>
        </div>
      </footer>
    </>
  );
};

export default BottomBar;
