// Minimal chart.js line-chart wrapper. Replaces solid-chartjs, which pins
// solid-js 1.x. Registers everything the CPU/RAM stat cards need, including
// the Filler plugin for area gradients (fill: true).

import { createEffect, onSettled } from "solid-js";
import {
  CategoryScale,
  Chart,
  Filler,
  Legend,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
  Tooltip,
  type ChartData,
  type ChartOptions,
} from "chart.js";

Chart.register(
  CategoryScale,
  Filler,
  Legend,
  LinearScale,
  LineController,
  LineElement,
  PointElement,
  Tooltip,
);

interface LineChartProps {
  data: ChartData<"line">;
  options: ChartOptions<"line">;
}

export default function LineChart(props: LineChartProps) {
  let canvasRef: HTMLCanvasElement | undefined;
  let chart: Chart<"line"> | undefined;

  onSettled(() => {
    if (!canvasRef) return;
    chart = new Chart(canvasRef, {
      type: "line",
      data: props.data,
      options: props.options,
    });
    return () => {
      chart?.destroy();
      chart = undefined;
    };
  });

  createEffect(
    () => ({ data: props.data, options: props.options }),
    ({ data, options }) => {
      if (!chart) return;
      chart.data = data;
      chart.options = options;
      chart.update("none");
    },
  );

  return <canvas ref={canvasRef} />;
}
