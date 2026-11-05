import { Show, createMemo, type Accessor } from "solid-js";
import {
  clientNow,
  formatIsoAge,
  formatUptimeMs,
  healthState,
  parseUptimeToMs,
} from "../state/health";
import { t } from "../state/i18n";
import BuildDetails, { Ltr } from "./BuildDetails";
import { postgresVersion } from "../utils/statusMetadata";

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
    // Labels are translated, so the block follows the interface direction; each
    // value is its own left-to-right island so versions and latencies keep their order.
    <div class="system-status-details font-mono tabular-nums text-[11px] text-ink">
      <div class="space-y-1">
        <BuildDetails />
      </div>
      <div class="mt-3 space-y-1">
        <Show
          when={healthState()}
          fallback={<div>{t("bottom_bar.metrics")}: …</div>}
        >
          {(health) => (
            <>
              <div>
                {t("bottom_bar.up")} <Ltr>{liveUptime()}</Ltr> ·{" "}
                {t("bottom_bar.handled")} <Ltr>{health().responses_handled}</Ltr>{" "}
                {t("bottom_bar.responses")} · {t("bottom_bar.sessions")}{" "}
                <Ltr>{health().users_logged_in}</Ltr>
              </div>
              <div>
                <Ltr>{postgresVersion(health().db_version)}</Ltr> ·{" "}
                {t("bottom_bar.db_latency")} <Ltr>{health().db_latency}</Ltr>
              </div>
              <div>
                {t("bottom_bar.time_to_report")}: <Ltr>{health().time_to_process ?? "?"}</Ltr>{" "}
                · {t("bottom_bar.net")}{" "}
                <Ltr>{health().client_latency_ms?.toFixed(1) ?? "?"}ms</Ltr> ·{" "}
                {t("bottom_bar.state_age")}{" "}
                <Ltr>{formatIsoAge(health().timestamp, clientNow())}</Ltr>
              </div>
            </>
          )}
        </Show>
      </div>
    </div>
  );
}
