import { onSettled } from "solid-js";

const TOP_BAR_SELECTOR = '[data-site-bar="top"]';
const BOTTOM_BAR_SELECTOR = '[data-site-bar="bottom"]';

/** Publishes the rendered fixed-bar sizes for viewport-bound mobile layouts. */
export function useSiteBarMeasurements(): void {
  onSettled(() => {
    const topBar = document.querySelector<HTMLElement>(TOP_BAR_SELECTOR);
    const bottomBar = document.querySelector<HTMLElement>(BOTTOM_BAR_SELECTOR);
    if (!topBar || !bottomBar) return;

    const update = () => {
      const top = Math.max(0, Math.ceil(topBar.getBoundingClientRect().bottom));
      const bottom = Math.max(
        0,
        Math.ceil(window.innerHeight - bottomBar.getBoundingClientRect().top),
      );
      document.documentElement.style.setProperty(
        "--site-header-height",
        `${top}px`,
      );
      document.documentElement.style.setProperty(
        "--site-footer-height",
        `${bottom}px`,
      );
    };

    update();
    const observer =
      typeof ResizeObserver === "undefined" ? null : new ResizeObserver(update);
    observer?.observe(topBar);
    observer?.observe(bottomBar);
    window.addEventListener("resize", update);
    window.visualViewport?.addEventListener("resize", update);
    return () => {
      observer?.disconnect();
      window.removeEventListener("resize", update);
      window.visualViewport?.removeEventListener("resize", update);
    };
  });
}
