import { createEffect, onCleanup, onSettled } from "solid-js";
import type { JSX } from "@solidjs/web";
import { motionEnabled } from "../../state/motion";
import type { ContourField } from "./contourField";

/** Decorative shader is skipped outright when the visitor asked to save data. */
export function shouldStartHeroField(win: Window = window): boolean {
  const connection = (win.navigator as Navigator & { connection?: { saveData?: boolean } })
    .connection;
  return connection?.saveData !== true;
}

/** Touch and narrow devices draw one still frame rather than a continuous loop. */
export function canAnimateHeroField(win: Window = window): boolean {
  const media = (query: string) => win.matchMedia?.(query).matches ?? false;
  return media("(min-width: 1024px)") && media("(pointer: fine)");
}

/**
 * Canvas behind the home hero. The shader chunk is imported only when the
 * browser is idle, so the initial route pays for this element and a few lines
 * of script. Without WebGL2 the canvas stays empty and the CSS grid behind it
 * remains. The animation switch starts and stops the loop while it is mounted;
 * switched off, the field holds a still frame.
 */
export default function HeroField(props: { pointerTarget: () => HTMLElement | undefined }): JSX.Element {
  let canvas: HTMLCanvasElement | undefined;
  let field: ContourField | null = null;
  let cancelled = false;
  let cancelStart: (() => void) | undefined;

  onSettled(() => {
    if (!shouldStartHeroField()) return;
    const start = () => {
      void import("./contourField").then(({ startContourField }) => {
        const target = props.pointerTarget();
        if (cancelled || !canvas || !target) return;
        field = startContourField(canvas, target, motionEnabled() && canAnimateHeroField());
        if (field) canvas.dataset.active = "";
      });
    };
    if (typeof window.requestIdleCallback === "function") {
      const handle = window.requestIdleCallback(start, { timeout: 2500 });
      cancelStart = () => window.cancelIdleCallback(handle);
    } else {
      const handle = window.setTimeout(start, 1200);
      cancelStart = () => window.clearTimeout(handle);
    }
  });

  createEffect(
    () => motionEnabled(),
    (enabled) => {
      field?.setAnimated(enabled && canAnimateHeroField());
    },
  );

  onCleanup(() => {
    cancelled = true;
    cancelStart?.();
    field?.dispose();
  });

  return <canvas ref={(element) => (canvas = element)} class="home-field" aria-hidden="true" />;
}
