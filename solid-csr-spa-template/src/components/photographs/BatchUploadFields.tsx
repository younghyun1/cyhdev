// Multi-file upload as an Instagram-style swipe form: pick N files, then step
// through them one at a time (arrows / dots / touch swipe), filling a comment
// and a map location per photo before upload.
//
// A single Leaflet map instance is initialized lazily in the map div's ref
// callback (the div only mounts once files are selected, so the previous
// onMount-at-component-mount approach saw an undefined ref and never created the
// map). The map always targets the currently-visible photo. Submit is blocked
// until every photo has a comment and a location. Builds FormData as ordered
// `files` + a `meta` JSON array aligned to file order + `context`.

import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  untrack,
} from "solid-js";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import { GeoSearchControl, OpenStreetMapProvider } from "leaflet-geosearch";
import "leaflet-geosearch/dist/geosearch.css";
import { pageStyles } from "../../styles/pageStyles";
import { t, tx } from "../../state/i18n";

const MAX_FILES_PER_BATCH = 50;
const SWIPE_THRESHOLD_PX = 40;

interface FileEntry {
  id: string;
  file: File;
  previewUrl: string;
  comment: string;
  lat: number | null;
  lon: number | null;
}

interface BatchUploadFieldsProps {
  submitting: boolean;
  progress: number;
  onSubmit: (formData: FormData) => void;
  onCancel: () => void;
}

let entrySeq = 0;

const entryComplete = (e: FileEntry) =>
  e.comment.trim().length > 0 && e.lat !== null && e.lon !== null;

export default function BatchUploadFields(props: BatchUploadFieldsProps) {
  const [entries, setEntries] = createSignal<FileEntry[]>([]);
  const [index, setIndex] = createSignal(0);
  const [err, setErr] = createSignal<string | null>(null);

  let map: L.Map | null = null;
  let marker: L.Marker | null = null;
  let touchStartX = 0;

  const emojiIcon = L.divIcon({
    className: "emoji-marker",
    html: "📍",
    iconSize: [30, 30],
    iconAnchor: [15, 30],
  });

  const current = createMemo(() => {
    const list = entries();
    if (list.length === 0) return null;
    return list[Math.min(index(), list.length - 1)] ?? null;
  });
  const incompleteCount = createMemo(
    () => entries().filter((e) => !entryComplete(e)).length,
  );
  const allValid = createMemo(
    () => entries().length > 0 && incompleteCount() === 0,
  );

  const setLocationForCurrent = (lat: number, lon: number) => {
    const id = current()?.id;
    if (!id) return;
    setEntries((prev) =>
      prev.map((e) => (e.id === id ? { ...e, lat, lon } : e)),
    );
  };

  const updateComment = (comment: string) => {
    const id = current()?.id;
    if (!id) return;
    setEntries((prev) =>
      prev.map((e) => (e.id === id ? { ...e, comment } : e)),
    );
  };

  const handleFiles = (fileList: FileList | null) => {
    if (!fileList) return;
    const incoming = Array.from(fileList);
    if (entries().length + incoming.length > MAX_FILES_PER_BATCH) {
      setErr(tx("photos.batch_too_many", { max: MAX_FILES_PER_BATCH }));
      return;
    }
    setErr(null);
    const added: FileEntry[] = incoming.map((file) => ({
      id: `e${entrySeq++}`,
      file,
      previewUrl: URL.createObjectURL(file),
      comment: "",
      lat: null,
      lon: null,
    }));
    setEntries((prev) => [...prev, ...added]);
  };

  const removeCurrent = () => {
    const id = current()?.id;
    if (!id) return;
    // Compute the next list eagerly: signal reads do not observe queued
    // writes until the microtask flush, so clamp against the new length.
    const prev = entries();
    const target = prev.find((e) => e.id === id);
    if (target) URL.revokeObjectURL(target.previewUrl);
    const next = prev.filter((e) => e.id !== id);
    setEntries(next);
    setIndex((i) => Math.max(0, Math.min(i, next.length - 1)));
  };

  const go = (delta: number) => {
    const len = entries().length;
    if (len === 0) return;
    setIndex((i) => Math.max(0, Math.min(len - 1, i + delta)));
  };

  // Keep currentIndex in range as the list shrinks.
  createEffect(
    () => ({ len: entries().length, i: index() }),
    ({ len, i }) => {
      if (len > 0 && i > len - 1) setIndex(len - 1);
    },
  );

  // Sync the marker + recenter to the currently-visible photo.
  createEffect(
    () => current(),
    (e) => {
      if (!map) return;
      if (e && e.lat !== null && e.lon !== null) {
        if (marker) marker.setLatLng([e.lat, e.lon]);
        else marker = L.marker([e.lat, e.lon], { icon: emojiIcon }).addTo(map);
        map.setView([e.lat, e.lon], Math.max(map.getZoom() ?? 2, 6));
      } else if (marker) {
        marker.remove();
        marker = null;
      }
    },
  );

  // Tear the map down when all files are removed so a fresh div re-inits cleanly.
  createEffect(
    () => entries().length === 0,
    (empty) => {
      if (empty && map) {
        map.remove();
        map = null;
        marker = null;
      }
    },
  );

  const initMap = (el: HTMLDivElement) => {
    if (map) return;
    map = L.map(el).setView([20, 0], 2);
    L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution:
        '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
    }).addTo(map);

    const provider = new OpenStreetMapProvider();
    // @ts-expect-error GeoSearchControl constructor types do not match runtime usage.
    const searchControl = new GeoSearchControl({
      provider,
      style: "bar",
      autoClose: true,
      keepResult: true,
      showMarker: false,
    });
    map.addControl(searchControl);
    map.on("geosearch/showlocation", ((result: {
      location: { x: number; y: number };
    }) => {
      untrack(() => setLocationForCurrent(result.location.y, result.location.x));
    }) as unknown as L.LeafletEventHandlerFn);
    map.on("click", (e) =>
      untrack(() => setLocationForCurrent(e.latlng.lat, e.latlng.lng)),
    );

    // The map mounts inside a modal; fix tile layout once the panel has sized.
    setTimeout(() => map?.invalidateSize(), 50);
    const e = current();
    if (e && e.lat !== null && e.lon !== null) {
      marker = L.marker([e.lat, e.lon], { icon: emojiIcon }).addTo(map);
      map.setView([e.lat, e.lon], 6);
    }
  };

  onCleanup(() => {
    for (const e of entries()) URL.revokeObjectURL(e.previewUrl);
    map?.remove();
    map = null;
  });

  const handleSubmit = (e: Event) => {
    e.preventDefault();
    if (!allValid() || props.submitting) {
      setErr(t("photos.batch_needs_meta"));
      return;
    }
    const formData = new FormData();
    const meta = entries().map((entry) => ({
      comment: entry.comment,
      lat: entry.lat,
      lon: entry.lon,
    }));
    for (const entry of entries()) formData.append("files", entry.file);
    formData.append("meta", JSON.stringify(meta));
    formData.append("context", "photography");
    props.onSubmit(formData);
  };

  let addInput: HTMLInputElement | undefined;

  return (
    <form onSubmit={handleSubmit} class="flex flex-col gap-4">
      <h2 class={pageStyles.sectionTitle}>{t("photos.batch_upload_title")}</h2>

      <Show when={err()}>
        <p class={pageStyles.alertError}>{err()}</p>
      </Show>

      <Show
        when={entries().length > 0}
        fallback={
          <label class="flex flex-col items-center justify-center gap-2 rounded-sm border-2 border-dashed border-line px-6 py-12 cursor-pointer hover:border-line-strong transition">
            <span class={`text-sm font-medium ${pageStyles.muted}`}>
              {t("photos.select_images")}
            </span>
            <input
              type="file"
              accept="image/*"
              multiple
              class="hidden"
              onChange={(e) => {
                handleFiles(e.currentTarget.files);
                e.currentTarget.value = "";
              }}
            />
          </label>
        }
      >
        {/* Position + dots */}
        <div class="flex flex-col gap-2">
          <div class="flex items-center justify-between">
            <span class={`text-sm font-medium ${pageStyles.muted}`}>
              {tx("photos.batch_photo_position", {
                current: index() + 1,
                total: entries().length,
              })}
            </span>
            <span
              class={
                entryComplete(current()!)
                  ? `${pageStyles.chipBase} ${pageStyles.chipCompleted}`
                  : `${pageStyles.chipBase} ${pageStyles.chipQueued}`
              }
            >
              {entryComplete(current()!) ? "✓" : "•"}
            </span>
          </div>
          <div class="flex gap-1.5 overflow-x-auto pb-1">
            <For each={entries()}>
              {(e, i) => (
                <button
                  type="button"
                  onClick={() => setIndex(i())}
                  title={e.file.name}
                  class={[
                    "h-2.5 w-2.5 shrink-0 rounded-full transition",
                    i() === index()
                      ? "bg-ink ring-2 ring-offset-1 ring-line-strong"
                      : entryComplete(e)
                        ? "bg-ok"
                        : "bg-line-strong",
                  ]}
                />
              )}
            </For>
          </div>
        </div>

        {/* Swipeable image */}
        <div
          class="relative flex items-center justify-center bg-surface-2 rounded-sm overflow-hidden select-none"
          style={{ height: "260px" }}
          onTouchStart={(e) => (touchStartX = e.touches[0]?.clientX ?? 0)}
          onTouchEnd={(e) => {
            const dx = (e.changedTouches[0]?.clientX ?? 0) - touchStartX;
            if (Math.abs(dx) > SWIPE_THRESHOLD_PX) go(dx < 0 ? 1 : -1);
          }}
        >
          <img
            src={current()!.previewUrl}
            alt={current()!.file.name}
            class="max-h-full max-w-full object-contain"
          />
          <Show when={index() > 0}>
            <button
              type="button"
              class="absolute left-2 top-1/2 -translate-y-1/2 h-9 w-9 flex items-center justify-center rounded-full bg-black/50 hover:bg-black/70 text-white text-2xl leading-none transition"
              onClick={() => go(-1)}
              aria-label={t("photos.previous")}
            >
              ‹
            </button>
          </Show>
          <Show when={index() < entries().length - 1}>
            <button
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 h-9 w-9 flex items-center justify-center rounded-full bg-black/50 hover:bg-black/70 text-white text-2xl leading-none transition"
              onClick={() => go(1)}
              aria-label={t("photos.next")}
            >
              ›
            </button>
          </Show>
        </div>

        <p class="text-xs font-medium truncate" title={current()!.file.name}>
          {current()!.file.name}
        </p>

        {/* Comment for current */}
        <textarea
          value={current()!.comment}
          onInput={(e) => updateComment(e.currentTarget.value)}
          rows={2}
          class={pageStyles.textarea}
          placeholder={t("photos.comment_placeholder")}
        />

        {/* Location for current */}
        <div>
          <label class={`block text-sm font-medium ${pageStyles.muted} mb-1`}>
            {t("photos.location")}
          </label>
          <div ref={(el) => initMap(el)} class="map-container" />
          <p class="mt-1 text-xs text-ink-muted">
            <Show
              when={current()!.lat !== null && current()!.lon !== null}
              fallback={t("photos.select_location")}
            >
              {tx("photos.selected", {
                lat: current()!.lat?.toFixed(5) ?? "",
                lon: current()!.lon?.toFixed(5) ?? "",
              })}
            </Show>
          </p>
        </div>

        <div class="flex items-center justify-between gap-2">
          <button
            type="button"
            class={pageStyles.buttonGhost}
            onClick={() => addInput?.click()}
          >
            + {t("photos.batch_add_more")}
          </button>
          <button
            type="button"
            class={pageStyles.buttonGhost}
            onClick={removeCurrent}
          >
            {t("photos.batch_remove")}
          </button>
          <input
            ref={(el) => (addInput = el)}
            type="file"
            accept="image/*"
            multiple
            class="hidden"
            onChange={(e) => {
              handleFiles(e.currentTarget.files);
              e.currentTarget.value = "";
            }}
          />
        </div>

        <Show when={incompleteCount() > 0}>
          <p class="text-xs text-accent">
            {tx("photos.batch_incomplete", { count: incompleteCount() })}
          </p>
        </Show>
      </Show>

      <Show when={props.submitting}>
        <div class="w-full mt-2">
          <div class="w-full bg-surface-2 rounded-sm h-2 overflow-hidden">
            <div
              class="bg-ink h-2 transition-all duration-150"
              style={{ width: `${props.progress}%` }}
            />
          </div>
          <p class={`mt-1 text-xs ${pageStyles.muted} text-right`}>
            {props.progress}%
          </p>
        </div>
      </Show>

      <div class="flex justify-end gap-2 mt-2">
        <button
          type="button"
          onClick={() => props.onCancel()}
          class={pageStyles.buttonSecondary}
        >
          {t("common.cancel")}
        </button>
        <button
          type="submit"
          disabled={!allValid() || props.submitting}
          class={pageStyles.buttonPrimary}
        >
          {props.submitting ? t("common.uploading") : t("common.upload")}
        </button>
      </div>
    </form>
  );
}
