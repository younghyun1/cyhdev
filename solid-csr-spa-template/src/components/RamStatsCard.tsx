import LineChart from "./LineChart";
import { createMemo } from "solid-js";
import type {
  ChartOptions,
  ChartData,
  TooltipItem,
} from "chart.js";
import { createChartPalette } from "./chartPalette";
import { createTimeLabels } from "./chartTimeLabels";
import type { HostStatPoint } from "../dtos/shared/host_stats";
import { t } from "../state/i18n";
import { createMediaQuery } from "../utils/mediaQuery";

function formatMem(bytes: number): string {
  if (bytes < 1024) return `${bytes.toFixed(0)} B`;
  if (bytes < 1024 * 1024) return `${(bytes / 1024).toFixed(1)} KiB`;
  if (bytes < 1024 * 1024 * 1024)
    return `${(bytes / (1024 * 1024)).toFixed(1)} MiB`;
  return `${(bytes / (1024 * 1024 * 1024)).toFixed(1)} GiB`;
}

export default function RamStatsCard(props: {
  data: HostStatPoint[];
  limit: number;
}) {
  const isMobile = createMediaQuery("(max-width: 767px)");

  const C = createChartPalette();

  function padToLimit<T>(arr: T[], filler: T | null = null): (T | null)[] {
    const padLen = Math.max(0, props.limit - arr.length);
    return [...arr, ...Array(padLen).fill(filler)];
  }

  const latest = () => props.data[props.data.length - 1];

  // Round the y-axis ceiling off mem_total; only changes when total RAM changes,
  // so options identity stays stable across per-tick data updates.
  const yMax = createMemo(() => {
    const l = props.data[props.data.length - 1];
    return l ? Math.ceil(l.memT / (1024 * 1024) / 100) * 100 : undefined;
  });

  const labels = createTimeLabels(
    () => props.data,
    () => props.limit,
  );

  const chartData = (): ChartData<"line"> => ({
    labels: labels(),
    datasets: [
      {
        label: `${t("stats.memory_usage")} (MiB)`,
        data: padToLimit(
          props.data.map((s) => (s.memT - s.memF) / (1024 * 1024)),
        ),
        fill: true,
        backgroundColor: C().fill,
        borderColor: C().series,
        borderWidth: 2,
        tension: 0.4,
        pointRadius: 0,
      },
    ],
  });

  // Depends only on theme tokens and viewport, so the chart keeps one options
  // object across streamed samples instead of rebuilding it every second.
  const chartOptions = createMemo((): ChartOptions<"line"> => ({
    animation: false,
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      x: {
        grid: { color: C().border },
        ticks: { color: C().font, maxTicksLimit: isMobile() ? 4 : 11 },
      },
      y: {
        beginAtZero: true,
        grid: { color: C().border },
        ticks: { color: C().font, maxTicksLimit: isMobile() ? 5 : 11 },
        max: yMax(),
      },
    },
    plugins: {
      legend: {
        display: !isMobile(),
        labels: { color: C().font },
      },
      tooltip: {
        callbacks: {
          label: (ctx: TooltipItem<"line">) => {
            const v = ctx.parsed.y;
            return `${ctx.dataset.label}: ${(v ?? 0).toFixed(1)} MiB`;
          },
        },
        backgroundColor: C().bg,
        titleColor: C().font,
        bodyColor: C().font,
        borderColor: C().border,
        borderWidth: 1,
      },
    },
  }));

  return (
    <div class="stats-chart-card flex flex-col gap-2">
      <div class="stats-metric-heading flex items-center justify-between px-1">
        <div class="flex items-center gap-2">
          <div class="stats-chart-dot" />
          <h3
            class="text-sm font-mono font-bold uppercase tracking-wider text-ink-muted"
          >
            {t("stats.memory_usage")}
          </h3>
        </div>
        <div class="stats-chart-value text-xl font-bold font-mono text-right tabular-nums">
          {latest()
            ? `${formatMem(latest()!.memT - latest()!.memF)} / ${formatMem(
                latest()!.memT,
              )}`
            : "--/--"}
        </div>
      </div>

      <div class="stats-chart">
        <div class="absolute inset-0 p-4">
          <LineChart data={chartData()} options={chartOptions()} label={t("stats.memory_usage")} />
        </div>
      </div>
    </div>
  );
}
