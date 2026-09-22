import LineChart from "./LineChart";
import type { ChartOptions, ChartData } from "chart.js";
import { createChartPalette } from "./chartPalette";
import type { HostStatPoint } from "../dtos/shared/host_stats";
import { t } from "../state/i18n";
import { createMediaQuery } from "../utils/mediaQuery";

export default function CpuStatsCard(props: {
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

  const labels = () => {
    const s = props.data;
    const blanks = Array(Math.max(0, props.limit - s.length)).fill("");
    const times = s.map((s) =>
      new Date(s.ts).toLocaleTimeString(undefined, { hour12: false }),
    );
    return [...times, ...blanks];
  };

  const chartData = (): ChartData<"line"> => ({
    labels: labels(),
    datasets: [
      {
        label: `${t("stats.cpu_usage")} (%)`,
        data: padToLimit(props.data.map((s) => s.cpu)),
        fill: true,
        backgroundColor: C().fill,
        borderColor: C().series,
        borderWidth: 2,
        tension: 0.4,
        pointRadius: 0,
        yAxisID: "y",
      },
    ],
  });

  const chartOptions = (): ChartOptions<"line"> => ({
    animation: false,
    responsive: true,
    maintainAspectRatio: false,
    scales: {
      x: {
        grid: { color: C().border },
        ticks: { color: C().font, maxTicksLimit: isMobile() ? 4 : 11 },
      },
      y: {
        min: 0,
        max: 100,
        grid: { color: C().border },
        ticks: { color: C().font, maxTicksLimit: isMobile() ? 5 : 11 },
      },
    },
    plugins: {
      legend: {
        display: !isMobile(),
        labels: { color: C().font },
      },
      tooltip: {
        backgroundColor: C().bg,
        titleColor: C().font,
        bodyColor: C().font,
        borderColor: C().border,
        borderWidth: 1,
      },
    },
  });

  return (
    <div class="stats-chart-card flex flex-col gap-2">
      <div class="stats-metric-heading flex items-center justify-between px-1">
        <div class="flex items-center gap-2">
          <div class="stats-chart-dot" />
          <h3
            class="text-sm font-mono font-bold uppercase tracking-wider text-ink-muted"
          >
            {t("stats.cpu_usage")}
          </h3>
        </div>
        <div class="stats-chart-value text-xl font-mono font-bold tabular-nums">
          {latest() ? latest()!.cpu.toFixed(1) + "%" : "--%"}
        </div>
      </div>

      <div class="stats-chart">
        <div class="absolute inset-0 p-4">
          <LineChart data={chartData()} options={chartOptions()} label={t("stats.cpu_usage")} />
        </div>
      </div>
    </div>
  );
}
