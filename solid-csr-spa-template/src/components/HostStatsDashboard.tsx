import { onSettled, onCleanup, createMemo, createSignal, Show } from "solid-js";
import { theme } from "../state/theme";
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
  const isDark = () => theme() === "dark";

  // simple palette helper
  const C = () => ({
    bg: isDark() ? "#1f2937" : "#fff",
    border: isDark() ? "#374151" : "#d1d5db",
    font: isDark() ? "#e2e8f0" : "#334155",
    cardBg: isDark() ? "#111827" : "#f3f4f6",
  });

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
    (import.meta.env.VITE_API_URL || "")
      .replace(/^http/, "ws")
      .replace(/\/$/, "") + "/ws/host-stats";

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
        <div
          class="p-2 mb-2 rounded-sm text-xs font-mono max-w-xs"
          style={{
            background: isDark() ? "#7f1d1d" : "#fee2e2",
            color: isDark() ? "#fee2e2" : "#b91c1c",
          }}
        >
          {error()}
        </div>
      </Show>

      <div class="stats-dashboard w-full max-w-7xl mx-auto space-y-6">
        {/* Backend Health Stats Panel */}
        <div
          class="stats-panel p-6 rounded-sm shadow-lg border-2"
          style={{
            background: isDark()
              ? "linear-gradient(135deg, #1f2937 0%, #111827 100%)"
              : "linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)",
            "border-color": isDark() ? "#f59e0b" : "#b45309",
          }}
        >
          <div
            class="flex items-center gap-3 mb-4 pb-2 border-b border-opacity-20"
            style={{ "border-color": C().border }}
          >
            <div class="flex items-center justify-between w-full">
              <h2
                class="text-lg font-bold tracking-wide"
                style={{ color: C().font }}
              >
                {t("stats.server_stats")}
              </h2>
              <button
                class="px-3 py-1 text-xs font-semibold rounded-sm hover:opacity-80 transition-opacity"
                style={{
                  background: C().cardBg,
                  color: C().font,
                  border: `1px solid ${C().border}`,
                }}
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
                  <div
                    class="p-4 rounded-sm bg-opacity-50"
                    style={{ background: C().cardBg }}
                  >
                    <div class="text-xs opacity-70 mb-1">
                      {t("stats.uptime")}
                    </div>
                    <div class="text-xl font-mono font-bold tabular-nums">
                      {liveUptime()}
                    </div>
                  </div>
                  <div
                    class="p-4 rounded-sm bg-opacity-50"
                    style={{ background: C().cardBg }}
                  >
                    <div class="text-xs opacity-70 mb-1">
                      {t("stats.responses_handled")}
                    </div>
                    <div class="text-xl font-mono font-bold tabular-nums">
                      {hs.responses_handled.toLocaleString()}
                    </div>
                  </div>
                  <div
                    class="p-4 rounded-sm bg-opacity-50"
                    style={{ background: C().cardBg }}
                  >
                    <div class="text-xs opacity-70 mb-1">
                      {t("stats.active_sessions")}
                    </div>
                    <div class="text-xl font-mono font-bold tabular-nums">
                      {hs.users_logged_in}
                    </div>
                  </div>
                  <div
                    class="p-4 rounded-sm bg-opacity-50"
                    style={{ background: C().cardBg }}
                  >
                    <div class="text-xs opacity-70 mb-1">
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
        <div
          class="stats-panel p-6 rounded-sm shadow-lg border-2"
          style={{
            background: isDark()
              ? "linear-gradient(135deg, #1f2937 0%, #111827 100%)"
              : "linear-gradient(135deg, #ffffff 0%, #f8fafc 100%)",
            "border-color": isDark() ? "#f59e0b" : "#b45309",
          }}
        >
          <div
            class="stats-panel-heading flex items-center gap-3 mb-6 pb-2 border-b"
            style={{ "border-color": C().border }}
          >
            <div
              class="w-3 h-3 rounded-full animate-pulse"
              style={{ background: isDark() ? "#10b981" : "#059669" }}
            />
            <h2
              class="text-lg font-bold tracking-wide"
              style={{ color: C().font }}
            >
              {t("stats.live_host_metrics")}
            </h2>
            <div class="flex-1" />
            <div class="text-xs opacity-60" style={{ color: C().font }}>
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
