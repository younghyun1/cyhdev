import { createSignal, onSettled } from "solid-js";

/** Resolve CSS tokens once per theme, not once per streamed data point. */
export function createChartPalette() {
  const readPalette = () => {
    const styles = getComputedStyle(document.documentElement);
    const token = (name: string) => styles.getPropertyValue(name).trim();
    return {
      bg: token("--surface"),
      border: token("--line"),
      font: token("--ink-muted"),
      series: token("--accent"),
      fill: token("--accent-soft"),
    };
  };
  const [palette, setPalette] = createSignal(readPalette());
  onSettled(() => {
    // Read after the root class changes so canvas never lags one theme behind.
    const observer = new MutationObserver(() => setPalette(readPalette()));
    observer.observe(document.documentElement, { attributes: true, attributeFilter: ["class"] });
    const mobile = window.matchMedia("(max-width: 767px)");
    const update = () => setPalette(readPalette());
    mobile.addEventListener("change", update);
    setPalette(readPalette());
    return () => {
      observer.disconnect();
      mobile.removeEventListener("change", update);
    };
  });
  return palette;
}
