import { onSettled, onCleanup, createMemo, createSignal, Show } from "solid-js";
import "../styles/statistics.css";
import {
  healthState,
  clientNow,
  parseUptimeToMs,
  formatUptimeMs,
  refreshHealthState,
} from "../state/health";
import CpuStatsCard from "./CpuStatsCard";
import RamStatsCard from "./RamStatsCard";
import type { HostStatsRaw, HostStatPoint } from "../dtos/shared/host_stats";
import { t, tx } from "../state/i18n";
import { createMediaQuery } from "../utils/mediaQuery";
import { webSocketUrl } from "../utils/webSocketUrl";
import { API_URL } from "../services/api";

// parse exactly 20 bytes: [f32][u64][u64] (all big-endian)
function parseHostStats(buf: ArrayBuffer): HostStatsRaw | null {
  if (buf.byteLength !== 20) return null;
  const dv = new DataView(buf);
  const cpu_usage = dv.getFloat32(0, false);
  const mem_total = Number(dv.getBigUint64(4, false));
  const mem_free = Number(dv.getBigUint64(12, false));
  return { cpu_usage, mem_total, mem_free };
}

const HISTORY_LIMIT = 60;

export default function HostStatsDashboard(props: {
  wsUrl?: string;
}) {
  const [history, setHistory] = createSignal<HostStatPoint[]>([]);
  const [error, setError] = createSignal<string | null>(null);
  const isMobile = createMediaQuery("(max-width: 767px)");

  const liveUptime = createMemo(() => {
    const hs = healthState();
    if (!hs) return "–";

    const baselineMs =
      hs.baseline_uptime_ms ?? parseUptimeToMs(hs.server_uptime);
    const baselineTs = hs.baseline_timestamp ?? hs.timestamp;
    if (baselineMs == null || !baselineTs) return hs.server_uptime;

    const base = new Date(baselineTs);
    const now = clientNow() ?? new Date();
    const extra = now.getTime() - base.getTime();
    const totalMs =
      baselineMs + (Number.isFinite(extra) ? Math.max(extra, 0) : 0);
    return formatUptimeMs(totalMs);
  });

  let ws: WebSocket | null = null;
  let retry = 0;
  let reconnectTimer: ReturnType<typeof setTimeout> | undefined;
  let disposed = false;

  const wsUrl = () =>
    props.wsUrl ||
    webSocketUrl("/ws/host-stats", API_URL, window.location.origin);

  const scheduleReconnect = () => {
    if (
      disposed ||
      reconnectTimer !== undefined ||
      (isMobile() && document.hidden)
    ) {
      return;
    }
    const delay = Math.min(1000 * 2 ** retry, 15000);
    retry += 1;
    reconnectTimer = setTimeout(() => {
      reconnectTimer = undefined;
      connect();
    }, delay);
  };

  const connect = () => {
    if (
      disposed ||
      (isMobile() && document.hidden) ||
      ws?.readyState === WebSocket.OPEN ||
      ws?.readyState === WebSocket.CONNECTING
    ) {
      return;
    }
    try {
      ws = new WebSocket(wsUrl());
      ws.binaryType = "arraybuffer";
    } catch (e: unknown) {
      setError(tx("stats.ws_open_failed", { error: String(e) }));
      scheduleReconnect();
      return;
    }

    ws.onopen = () => {
      retry = 0;
      setError(null);
    };

    ws.onmessage = (evt) => {
      const raw = parseHostStats(evt.data as ArrayBuffer);
      if (!raw) {
        setError(t("stats.malformed_packet"));
        return;
      }
      setError(null);
      setHistory((old) => {
        const next: HostStatPoint = {
          ts: Date.now(),
          cpu: raw.cpu_usage,
          memT: raw.mem_total,
          memF: raw.mem_free,
        };
        const arr =
          old.length < HISTORY_LIMIT ? [...old, next] : [...old.slice(1), next];
        return arr;
      });
    };

    ws.onerror = () => setError(t("stats.websocket_error"));
    ws.onclose = () => {
      ws = null;
      setError((e) => e || t("stats.websocket_closed"));
      scheduleReconnect();
    };
  };

  onSettled(connect);

  function handleVisibility() {
    if (!isMobile()) return;
    if (document.hidden) {
      if (reconnectTimer !== undefined) {
        clearTimeout(reconnectTimer);
        reconnectTimer = undefined;
      }
      const current = ws;
      ws = null;
      if (current) {
        current.onclose = null;
        current.close();
      }
      return;
    }
    connect();
  }

  onSettled(() => {
    document.addEventListener("visibilitychange", handleVisibility);
  });

  onCleanup(() => {
    disposed = true;
    document.removeEventListener("visibilitychange", handleVisibility);
    if (reconnectTimer !== undefined) {
      clearTimeout(reconnectTimer);
      reconnectTimer = undefined;
    }
    ws?.close();
  });

  return (
    <div class="text-ink">
      <Show when={error()}>
        <div class="stats-error" role="status">
          {error()}
        </div>
      </Show>

      <div class="stats-dashboard w-full max-w-7xl mx-auto space-y-6">
        {/* Backend Health Stats Panel */}
        <div class="stats-panel">
          <div class="stats-panel-header">
            <div class="flex items-center justify-between w-full">
              <h2 class="stats-panel-title">
                {t("stats.server_stats")}
              </h2>
              <button
                class="stats-refresh"
                onClick={() => void refreshHealthState()}
              >
                {t("common.refresh")}
              </button>
            </div>
          </div>
          <div class="grid grid-cols-1 sm:grid-cols-2 xl:grid-cols-4 gap-4">
            <Show
              when={healthState()}
              keyed
              fallback={
                <div class="col-span-full">{t("stats.loading_health")}</div>
              }
            >
              {(hs) => (
                <>
                  <div class="stats-summary">
                    <div class="stats-summary-label">
                      {t("stats.uptime")}
                    </div>
                    <div class="text-xl font-mono font-bold tabular-nums">
                      {liveUptime()}
                    </div>
                  </div>
                  <div class="stats-summary">
                    <div class="stats-summary-label">
                      {t("stats.responses_handled")}
                    </div>
                    <div class="text-xl font-mono font-bold tabular-nums">
                      {hs.responses_handled.toLocaleString()}
                    </div>
                  </div>
                  <div class="stats-summary">
                    <div class="stats-summary-label">
                      {t("stats.active_sessions")}
                    </div>
                    <div class="text-xl font-mono font-bold tabular-nums">
                      {hs.users_logged_in}
                    </div>
                  </div>
                  <div class="stats-summary">
                    <div class="stats-summary-label">
                      {t("bottom_bar.db_latency")}
                    </div>
                    <div class="text-xl font-mono font-bold tabular-nums">
                      {hs.db_latency}
                    </div>
                  </div>
                </>
              )}
            </Show>
          </div>
        </div>

        {/* Live Host Stats Panel */}
        <div class="stats-panel">
          <div class="stats-panel-heading stats-panel-header">
            <div
              class={["stats-connection-dot", { "is-live": history().length > 0 && !error() }]}
              aria-hidden="true"
            />
            <h2 class="stats-panel-title">
              {t("stats.live_host_metrics")}
            </h2>
            <div class="flex-1" />
            <div class="stats-note">
              {t("stats.realtime_note")}
            </div>
          </div>

          <div class="flex flex-col gap-6">
            <CpuStatsCard data={history()} limit={HISTORY_LIMIT} />
            <RamStatsCard data={history()} limit={HISTORY_LIMIT} />
          </div>
        </div>
      </div>
    </div>
  );
}
