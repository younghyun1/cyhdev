import { createMemo, createSignal, For, Show, type Component } from "solid-js";

import type { ArchiveHistogramBucket } from "../../services/contracts/kcna_archive";
import { locale, t } from "../../state/i18n";
import { createMediaQuery } from "../../utils/mediaQuery";
import { formatCount, formatMonth, formatYear } from "../../utils/kcna_archive/format";
import {
  barAt,
  barHeight,
  inRange,
  monthBars,
  rangeOf,
  yearTicks,
  type MonthBar,
} from "../../utils/kcna_archive/histogram";

interface HistogramProps {
  readonly buckets: ReadonlyArray<ArchiveHistogramBucket>;
  readonly from: string | null;
  readonly to: string | null;
  /** Called with the chosen range, or `null` to clear it. */
  readonly onSelect: (range: { from: string; to: string } | null) => void;
}

/** Year labels that fit the axis; a phone fits fewer than a desktop. */
const AXIS_LABELS = 12;
const COMPACT_AXIS_LABELS = 5;

/**
 * Documents per month as bars, with a draggable span. Drawing, dragging, and
 * the axis are left to right in every interface language: time reads that way
 * on a chart, as on the statistics charts.
 */
const Histogram: Component<HistogramProps> = (props) => {
  const bars = createMemo(() => monthBars(props.buckets));
  const maximum = createMemo(() => bars().reduce((most, bar) => Math.max(most, bar.count), 0));
  const compact = createMediaQuery("(max-width: 767px)");
  const ticks = createMemo(() => yearTicks(bars(), compact() ? COMPACT_AXIS_LABELS : AXIS_LABELS));
  const [hover, setHover] = createSignal<MonthBar>();
  const [drag, setDrag] = createSignal<{ anchor: MonthBar; head: MonthBar }>();
  let chart: HTMLDivElement | undefined;

  const barUnder = (event: PointerEvent): MonthBar | undefined => {
    if (chart === undefined) return undefined;
    const box = chart.getBoundingClientRect();
    if (box.width === 0) return undefined;
    return barAt(bars(), (event.clientX - box.left) / box.width);
  };

  // While dragging the pending span is shown; otherwise the applied range.
  const span = createMemo(() => {
    const active = drag();
    if (active !== undefined) return rangeOf(active.anchor, active.head);
    return props.from === null && props.to === null ? undefined : { from: props.from, to: props.to };
  });

  const position = (bar: MonthBar): number => {
    const first = bars()[0];
    return first === undefined ? 0 : (bar.month - first.month) / bars().length;
  };

  /** Left edge and width of the selected span, as shares of the chart. */
  const overlay = createMemo(() => {
    const current = span();
    if (current === undefined) return undefined;
    const selected = bars().filter((bar) => inRange(bar, current.from, current.to));
    const first = selected[0];
    const last = selected[selected.length - 1];
    if (first === undefined || last === undefined) return undefined;
    return { left: position(first), width: position(last) - position(first) + 1 / bars().length };
  });

  const onPointerDown = (event: PointerEvent): void => {
    const bar = barUnder(event);
    if (bar === undefined || event.button !== 0) return;
    chart?.setPointerCapture(event.pointerId);
    setDrag({ anchor: bar, head: bar });
  };

  const onPointerMove = (event: PointerEvent): void => {
    const bar = barUnder(event);
    setHover(bar);
    const active = drag();
    if (active !== undefined && bar !== undefined && bar.month !== active.head.month) {
      setDrag({ anchor: active.anchor, head: bar });
    }
  };

  const onPointerUp = (): void => {
    const active = drag();
    if (active === undefined) return;
    setDrag(undefined);
    const range = rangeOf(active.anchor, active.head);
    // Clicking the single selected month again clears the range.
    const same = range.from === props.from && range.to === props.to;
    props.onSelect(same ? null : range);
  };

  return (
    <Show when={bars().length > 0}>
      <section class="kcna-histogram" aria-label={t("kcna.histogram.label")}>
        <div
          ref={chart}
          dir="ltr"
          class={["kcna-histogram-chart", { "is-dragging": drag() !== undefined }]}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={() => setDrag(undefined)}
          onPointerLeave={() => setHover(undefined)}
        >
          <svg
            viewBox={`0 0 ${bars().length} 100`}
            preserveAspectRatio="none"
            role="img"
            aria-label={t("kcna.histogram.label")}
          >
            <For each={bars()}>
              {(bar, index) => (
                <rect
                  class={[
                    "kcna-bar",
                    {
                      "is-outside":
                        span() !== undefined && !inRange(bar, span()?.from ?? null, span()?.to ?? null),
                    },
                  ]}
                  x={index() + 0.1}
                  y="0"
                  width="0.8"
                  height="100"
                  style={{ transform: `scaleY(${barHeight(bar.count, maximum())})` }}
                />
              )}
            </For>
          </svg>
          <Show when={overlay()}>
            {(box) => (
              <div
                class="kcna-histogram-span"
                style={{ left: `${box().left * 100}%`, width: `${box().width * 100}%` }}
              />
            )}
          </Show>
          <Show when={hover()}>
            {(bar) => (
              <div
                class="kcna-histogram-tip"
                style={{ left: `${(position(bar()) + 0.5 / bars().length) * 100}%` }}
              >
                <strong>{formatMonth(bar().start, locale())}</strong> {formatCount(bar().count, locale())}
              </div>
            )}
          </Show>
        </div>
        <div class="kcna-histogram-axis" dir="ltr" aria-hidden="true">
          <For each={ticks()}>
            {(bar) => (
              <span style={{ left: `${position(bar) * 100}%` }}>{formatYear(bar.start, locale())}</span>
            )}
          </For>
        </div>
      </section>
    </Show>
  );
};

export default Histogram;
