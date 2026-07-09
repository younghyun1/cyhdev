import type { Component } from "solid-js";
import {
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import { useLocation } from "@solidjs/router";
import {
  healthState,
  clientNow,
  setClientNow,
  refreshHealthState,
  parseUptimeToMs,
  formatUptimeMs,
  formatIsoAge,
} from "../state/health";
import { serverBuildInfo } from "../state/server_info";
import { t } from "../state/i18n";

// Expose build info injected by Vite
declare const __BUILD_TIMESTAMP__: string;
declare const __SOLID_VERSION__: string;

const BottomBar: Component = () => {
  const location = useLocation();

  const [detailsOpen, setDetailsOpen] = createSignal(false);
  const [isMobile, setIsMobile] = createSignal(false);

  onMount(() => {
    const update = () =>
      setIsMobile(window.matchMedia("(max-width: 639px)").matches);
    update();
    window.addEventListener("resize", update);
    onCleanup(() => window.removeEventListener("resize", update));
  });

  const closeDetails = () => setDetailsOpen(false);

  // Refresh health state on route changes
  createEffect(() => {
    // depend on pathname so this re-runs on each navigation
    void location.pathname; // depend on pathname so this re-runs on each navigation
    refreshHealthState();
  });

  // Live ticking for uptime / age display
  onMount(() => {
    const interval = setInterval(() => {
      setClientNow(new Date());
    }, 1000);

    onCleanup(() => clearInterval(interval));
  });

  const liveUptime = createMemo(() => {
    const hs = healthState();
    if (!hs) return "…";

    const baselineMs =
      hs.baseline_uptime_ms ?? parseUptimeToMs(hs.server_uptime);
    const baselineTs = hs.baseline_timestamp ?? hs.timestamp;

    let result: string = hs.server_uptime;

    if (baselineMs != null && baselineTs) {
      const base = new Date(baselineTs);
      const now = clientNow() ?? new Date();
      const extra = now.getTime() - base.getTime();
      const totalMs =
        baselineMs + (Number.isFinite(extra) ? Math.max(extra, 0) : 0);

      result = formatUptimeMs(totalMs);
    }

    return result;
  });

  const mobileSummary = createMemo(() => {
    const hs = healthState();
    if (!hs) return `${t("bottom_bar.site_status")}: …`;
    return `${t("bottom_bar.up")} ${liveUptime()} · ${hs.responses_handled} ${t("bottom_bar.responses")} · ${hs.users_logged_in} ${t("bottom_bar.sessions")}`;
  });

  const handleFooterClick = () => {
    if (isMobile()) setDetailsOpen(true);
  };

  return (
    <>
      {/* Mobile-only details modal */}
      <Show when={detailsOpen() && isMobile()}>
        <div
          class="fixed inset-0 z-60 flex items-end justify-center sm:hidden"
          onClick={(e) => {
            if (e.target === e.currentTarget) closeDetails();
          }}
        >
          <div class="absolute inset-0 bg-black/60" />
          <div class="relative w-full max-h-[75vh] overflow-y-auto rounded-t-sm bg-surface border-t border-line p-4 shadow-2xl">
            <div class="flex items-center justify-between gap-3">
              <div class="text-sm font-semibold text-ink">
                {t("bottom_bar.site_status")}
              </div>
              <button
                type="button"
                class="px-3 py-1.5 text-xs rounded-sm border border-line text-ink hover:bg-surface-2"
                onClick={closeDetails}
                aria-label={t("common.close")}
              >
                {t("common.close")}
              </button>
            </div>

            <div class="mt-3 space-y-3 font-mono tabular-nums text-[11px] text-ink">
              <div class="space-y-1">
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

              <div class="space-y-1">
                <Show
                  when={healthState()}
                  fallback={<div>{t("bottom_bar.metrics")}: …</div>}
                >
                  {(() => {
                    const hs = healthState()!;
                    return (
                      <>
                        <div>
                          {t("bottom_bar.up")} {liveUptime()} ·{" "}
                          {t("bottom_bar.handled")} {hs.responses_handled}{" "}
                          {t("bottom_bar.responses")} ·{" "}
                          {t("bottom_bar.sessions")} {hs.users_logged_in}
                        </div>
                        <div>
                          {t("bottom_bar.db")} {hs.db_version} ·{" "}
                          {t("bottom_bar.db_latency")} {hs.db_latency}
                        </div>
                        <div>
                          {t("bottom_bar.time_to_report")}:{" "}
                          {hs.time_to_process ?? "?"} · {t("bottom_bar.net")}{" "}
                          {hs.client_latency_ms?.toFixed(1) ?? "?"}ms ·{" "}
                          {t("bottom_bar.state_age")}{" "}
                          {formatIsoAge(hs.timestamp, clientNow())}
                        </div>
                      </>
                    );
                  })()}
                </Show>
              </div>
            </div>
          </div>
        </div>
      </Show>

      <footer
        class="fixed bottom-0 left-0 w-full transition-colors duration-90 border-t border-line bg-paper/90 backdrop-blur text-[9px] sm:text-[11px]"
        style={{
          "z-index": 50,
        }}
        onClick={handleFooterClick}
        role={isMobile() ? "button" : undefined}
        aria-label={isMobile() ? t("bottom_bar.open_details") : undefined}
        tabIndex={isMobile() ? 0 : undefined}
        onKeyDown={(e) => {
          if (!isMobile()) return;
          if (e.key === "Enter" || e.key === " ") {
            e.preventDefault();
            setDetailsOpen(true);
          }
          if (e.key === "Escape") closeDetails();
        }}
      >
        <div class="w-full px-2 sm:px-3 py-0.5 sm:py-1.5 flex flex-row justify-between items-start gap-2 sm:gap-3 font-mono tabular-nums">
          {/* Desktop/tablet: keep full content */}
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
                const hs = healthState()!;
                return (
                  <>
                    <div>
                      {t("bottom_bar.up")} {liveUptime()} ·{" "}
                      {t("bottom_bar.handled")} {hs.responses_handled}{" "}
                      {t("bottom_bar.responses")} ·{" "}
                      {t("bottom_bar.sessions")} {hs.users_logged_in}
                    </div>

                    <div class="hidden xs:block sm:block">
                      {t("bottom_bar.db")} {hs.db_version} ·{" "}
                      {t("bottom_bar.db_latency")} {hs.db_latency}
                    </div>

                    <div class="hidden sm:block">
                      {t("bottom_bar.time_to_report")}:{" "}
                      {hs.time_to_process ?? "?"} · {t("bottom_bar.net")}{" "}
                      {hs.client_latency_ms?.toFixed(1) ?? "?"}ms ·{" "}
                      {t("bottom_bar.state_age")}{" "}
                      {formatIsoAge(hs.timestamp, clientNow())}
                    </div>
                  </>
                );
              })()
            ) : (
              <div>{t("bottom_bar.metrics")}: …</div>
            )}
          </div>

          {/* Mobile: compact summary + hint */}
          <div class="sm:hidden w-full flex items-center justify-between gap-2 text-ink-muted leading-tight">
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
