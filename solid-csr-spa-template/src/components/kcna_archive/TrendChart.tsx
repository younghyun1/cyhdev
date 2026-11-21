import { createMemo, For, type Component } from "solid-js";

import type { ArchiveTrendsResponse } from "../../services/contracts/kcna_archive";
import { locale, t } from "../../state/i18n";
import { formatCount, formatMonth, formatPercent, formatYear } from "../../utils/kcna_archive/format";
import { linePath, plotted, type TrendMode } from "../../utils/kcna_archive/trends";

const WIDTH = 1000;
const HEIGHT = 300;
/** Horizontal guide lines, including the baseline. */
const GUIDES = [0, 0.25, 0.5, 0.75, 1] as const;

interface TrendChartProps {
  readonly data: ArchiveTrendsResponse;
  readonly mode: TrendMode;
}

/**
 * Term frequencies as lines over time. Series differ by dash pattern as well
 * as by color, so the chart reads without color vision. Axes stay left to
 * right in every interface language.
 */
const TrendChart: Component<TrendChartProps> = (props) => {
  const values = createMemo(() => plotted(props.data, props.mode));
  const maximum = createMemo(() => Math.max(0, ...values().flat()));
  const label = (start: string): string =>
    props.data.bucket === "year" ? formatYear(start, locale()) : formatMonth(start, locale());
  const peak = (): string =>
    props.mode === "share" ? formatPercent(maximum(), locale()) : formatCount(maximum(), locale());

  return (
    <figure class="kcna-chart" dir="ltr">
      <div class="kcna-chart-plot">
        <svg viewBox={`0 0 ${WIDTH} ${HEIGHT}`} preserveAspectRatio="none" role="img" aria-label={t("kcna.trends.title")}>
          <For each={GUIDES}>
            {(guide) => (
              <line class="kcna-chart-guide" x1="0" x2={WIDTH} y1={guide * HEIGHT} y2={guide * HEIGHT} />
            )}
          </For>
          <For each={values()}>
            {(series, index) => (
              <path class={["kcna-line", `kcna-line--${index() + 1}`]} d={linePath(series, maximum(), WIDTH, HEIGHT)} />
            )}
          </For>
        </svg>
        <span class="kcna-chart-max">{peak()}</span>
      </div>
      <figcaption class="kcna-chart-caption">
        <span class="kcna-chart-range">
          {label(props.data.starts[0] ?? "")} – {label(props.data.starts[props.data.starts.length - 1] ?? "")}
        </span>
        <ul class="kcna-legend">
          <For each={props.data.series}>
            {(series, index) => (
              <li class="kcna-legend-item">
                <svg class="kcna-legend-swatch" viewBox="0 0 24 8" aria-hidden="true">
                  <line class={["kcna-line", `kcna-line--${index() + 1}`]} x1="0" x2="24" y1="4" y2="4" />
                </svg>
                <span class="kcna-legend-term" dir="auto" title={series.term}>
                  {series.term}
                </span>
              </li>
            )}
          </For>
        </ul>
      </figcaption>
    </figure>
  );
};

export default TrendChart;
