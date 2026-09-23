import {
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
} from "solid-js";
import { t } from "../../state/i18n";
import { createImagePreloader } from "./imagePreloader";

/** Delay before the loading line appears, so cached and fast loads never flash it. */
export const LOADING_INDICATOR_DELAY_MS = 150;
/** The current photo plus its previous and next neighbours. */
const PRELOAD_LIMIT = 3;

type StageStatus = "loading" | "ready" | "error";

interface PhotoStageImageProps {
  readonly src: string;
  readonly alt: string;
  /** Adjacent photo URLs, warmed once the current photo has settled. */
  readonly neighbors: readonly string[];
}

/**
 * Image area of the photograph viewer. The shown image is tied to the URL it
 * was decoded for, so when the photo changes the stage drops to its black
 * backdrop in the same render that updates the description; the previous
 * image is never left under new metadata. The new image fades in after
 * `decode()`, a thin loading line appears only when that takes longer than
 * `LOADING_INDICATOR_DELAY_MS`, and a failed load offers a retry.
 *
 * Renders as a fragment so the image stays a direct child of the stage, which
 * the mobile sizing rules target.
 */
export default function PhotoStageImage(props: PhotoStageImageProps) {
  const preloader = createImagePreloader({ maxEntries: PRELOAD_LIMIT });
  const [readySrc, setReadySrc] = createSignal<string | null>(null);
  const [failedSrc, setFailedSrc] = createSignal<string | null>(null);
  const [slowSrc, setSlowSrc] = createSignal<string | null>(null);
  const [attempt, setAttempt] = createSignal(0);

  // Derived from the current URL rather than written by an effect, so a
  // photo change hides the old image synchronously with the new metadata.
  const status = createMemo<StageStatus>(() => {
    if (readySrc() === props.src) return "ready";
    if (failedSrc() === props.src) return "error";
    return "loading";
  });
  const indicatorVisible = () =>
    status() === "loading" && slowSrc() === props.src;

  createEffect(
    () => ({ src: props.src, attempt: attempt() }),
    ({ src }) => {
      // Cleared when the photo changes or the viewer closes, so a superseded
      // or aborted load can never reveal its image or error.
      let current = true;
      const timer = setTimeout(() => {
        if (current) setSlowSrc(src);
      }, LOADING_INDICATOR_DELAY_MS);
      preloader
        .load(src, "high")
        .then(
          () => {
            if (current) setReadySrc(src);
          },
          () => {
            if (current) setFailedSrc(src);
          },
        )
        .finally(() => clearTimeout(timer));
      return () => {
        current = false;
        clearTimeout(timer);
      };
    },
  );

  // Keep only the current photo and its neighbours; start the neighbours
  // after the current photo settles so it gets the bandwidth first.
  createEffect(
    () => ({
      settled: status() !== "loading",
      src: props.src,
      neighbors: props.neighbors,
    }),
    ({ settled, src, neighbors }) => {
      preloader.retain([src, ...neighbors]);
      if (!settled) return;
      for (const url of neighbors) preloader.warm(url);
    },
  );

  onCleanup(() => preloader.clear());

  return (
    <>
      <Show when={status() === "ready"}>
        <img
          class="photo-stage-image"
          src={props.src}
          alt={props.alt}
          decoding="async"
        />
      </Show>
      <Show when={indicatorVisible()}>
        <div class="photo-stage-progress" data-photo-loading aria-hidden="true" />
      </Show>
      <span class="sr-only" role="status">
        {indicatorVisible() ? t("photos.image_loading") : ""}
      </span>
      <Show when={status() === "error"}>
        <div class="photo-stage-error" role="alert">
          <p>{t("photos.image_load_failed")}</p>
          <button
            type="button"
            class="photo-stage-retry"
            onClick={(event) => {
              event.stopPropagation();
              // Back to the loading state for the retry, not the stale error.
              setFailedSrc(null);
              setAttempt((count) => count + 1);
            }}
          >
            {t("photos.image_retry")}
          </button>
        </div>
      </Show>
    </>
  );
}
