import { onCleanup, onSettled } from "solid-js";
import type { JSX } from "@solidjs/web";

/** Conditions under which the decorative shader is worth starting. */
export function shouldStartHeroField(win: Window = window): boolean {
  const media = (query: string) => win.matchMedia?.(query).matches ?? false;
  const connection = (win.navigator as Navigator & { connection?: { saveData?: boolean } })
    .connection;
  return (
    media("(min-width: 1024px)") &&
    media("(pointer: fine)") &&
    !media("(prefers-reduced-motion: reduce)") &&
    connection?.saveData !== true
  );
}

/**
 * Canvas behind the home hero. The shader chunk is imported only when the
 * browser is idle and the device qualifies, so the initial route pays for
 * this element and a few lines of script. Without WebGL2 the canvas stays
 * empty and the CSS grid behind it remains.
 */
export default function HeroField(props: { pointerTarget: () => HTMLElement | undefined }): JSX.Element {
  let canvas: HTMLCanvasElement | undefined;
  let dispose: (() => void) | null = null;
  let cancelled = false;
  let idleHandle: number | undefined;

  onSettled(() => {
    if (!shouldStartHeroField()) return;
    const start = () => {
      void import("./contourField").then(({ startContourField }) => {
        const target = props.pointerTarget();
        if (cancelled || !canvas || !target) return;
        dispose = startContourField(canvas, target);
        if (dispose) canvas.dataset.active = "";
      });
    };
    if (typeof window.requestIdleCallback === "function") {
      idleHandle = window.requestIdleCallback(start, { timeout: 2500 });
    } else {
      idleHandle = window.setTimeout(start, 1200);
    }
  });

  onCleanup(() => {
    cancelled = true;
    if (idleHandle !== undefined) {
      if (typeof window.cancelIdleCallback === "function") window.cancelIdleCallback(idleHandle);
      window.clearTimeout(idleHandle);
    }
    dispose?.();
  });

  return <canvas ref={(element) => (canvas = element)} class="home-field" aria-hidden="true" />;
}
