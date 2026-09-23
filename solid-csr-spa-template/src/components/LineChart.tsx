// Minimal chart.js line-chart wrapper. Replaces solid-chartjs, which pins
// solid-js 1.x. Registers everything the CPU/RAM stat cards need, including
// the Filler plugin for shaded areas (fill: true).

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
  label: string;
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

  // Data streams every second while options change only with theme or
  // viewport, so each is applied on its own change and a streamed sample never
  // swaps in a new options object.
  createEffect(
    () => props.data,
    (data) => {
      if (!chart) return;
      chart.data = data;
      chart.update("none");
    },
  );
  createEffect(
    () => props.options,
    (options) => {
      if (!chart) return;
      chart.options = options;
      chart.update("none");
    },
  );

  return <canvas ref={canvasRef} role="img" aria-label={props.label} />;
}
