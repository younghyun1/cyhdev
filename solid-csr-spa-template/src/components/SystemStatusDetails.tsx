import { Show, createMemo, type Accessor } from "solid-js";
import {
  clientNow,
  formatIsoAge,
  formatUptimeMs,
  healthState,
  parseUptimeToMs,
} from "../state/health";
import { t } from "../state/i18n";
import { serverBuildInfo } from "../state/server_info";

declare const __BUILD_TIMESTAMP__: string;
declare const __SOLID_VERSION__: string;

export function createLiveUptime(): Accessor<string> {
  const uptime = createMemo(() => {
    const health = healthState();
    if (!health) return "…";
    const baseline =
      health.baseline_uptime_ms ?? parseUptimeToMs(health.server_uptime);
    const baselineTimestamp = health.baseline_timestamp ?? health.timestamp;
    if (baseline === null || !baselineTimestamp) return health.server_uptime;
    const elapsed =
      (clientNow() ?? new Date()).getTime() -
      new Date(baselineTimestamp).getTime();
    return formatUptimeMs(
      baseline + (Number.isFinite(elapsed) ? Math.max(0, elapsed) : 0),
    );
  });
  return uptime;
}

export default function SystemStatusDetails() {
  const liveUptime = createLiveUptime();

  return (
    <div class="system-status-details font-mono tabular-nums text-[11px] text-ink">
      <div class="space-y-1">
        <div>
          {t("bottom_bar.fe")}: {t("bottom_bar.built")} {__BUILD_TIMESTAMP__}{" "}
          {t("bottom_bar.with_solid")} {__SOLID_VERSION__}
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
      <div class="mt-3 space-y-1">
        <Show
          when={healthState()}
          fallback={<div>{t("bottom_bar.metrics")}: …</div>}
        >
          {(health) => (
            <>
              <div>
                {t("bottom_bar.up")} {liveUptime()} ·{" "}
                {t("bottom_bar.handled")} {health().responses_handled}{" "}
                {t("bottom_bar.responses")} · {t("bottom_bar.sessions")}{" "}
                {health().users_logged_in}
              </div>
              <div>
                {t("bottom_bar.db")} {health().db_version} ·{" "}
                {t("bottom_bar.db_latency")} {health().db_latency}
              </div>
              <div>
                {t("bottom_bar.time_to_report")}: {health().time_to_process ?? "?"}{" "}
                · {t("bottom_bar.net")}{" "}
                {health().client_latency_ms?.toFixed(1) ?? "?"}ms ·{" "}
                {t("bottom_bar.state_age")}{" "}
                {formatIsoAge(health().timestamp, clientNow())}
              </div>
            </>
          )}
        </Show>
      </div>
    </div>
  );
}
