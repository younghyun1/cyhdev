// Multi-file upload form with per-file mini-forms (comment + location).
//
// Collects N files, renders a row per file (object-URL thumbnail, comment, and a
// location picker that reuses a single shared Leaflet map targeting the active
// row). Submit is blocked until every file has a comment and a location. Builds
// FormData as ordered `files` + a `meta` JSON array aligned to file order +
// `context`, then hands it to the parent via `onSubmit`.

import {
  For,
  Show,
  createEffect,
  createMemo,
  createSignal,
  onCleanup,
  onMount,
} from "solid-js";
import L from "leaflet";
import "leaflet/dist/leaflet.css";
import { GeoSearchControl, OpenStreetMapProvider } from "leaflet-geosearch";
import "leaflet-geosearch/dist/geosearch.css";
import { pageStyles } from "../../styles/pageStyles";
import { t, tx } from "../../state/i18n";

const MAX_FILES_PER_BATCH = 50;

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

export default function BatchUploadFields(props: BatchUploadFieldsProps) {
  const [entries, setEntries] = createSignal<FileEntry[]>([]);
  const [activeId, setActiveId] = createSignal<string | null>(null);
  const [err, setErr] = createSignal<string | null>(null);

  let mapDiv: HTMLDivElement | undefined;
  let map: L.Map | null = null;
  let marker: L.Marker | null = null;

  const emojiIcon = L.divIcon({
    className: "emoji-marker",
    html: "📍",
    iconSize: [30, 30],
    iconAnchor: [15, 30],
  });

  const allValid = createMemo(() => {
    const list = entries();
    return (
      list.length > 0 &&
      list.every(
        (e) => e.comment.trim().length > 0 && e.lat !== null && e.lon !== null,
      )
    );
  });

  const setLocationForActive = (lat: number, lon: number) => {
    const id = activeId() ?? entries()[0]?.id ?? null;
    if (id === null) return;
    if (activeId() === null) setActiveId(id);
    setEntries((prev) =>
      prev.map((e) => (e.id === id ? { ...e, lat, lon } : e)),
    );
  };

  const handleFiles = (fileList: FileList | null) => {
    if (!fileList) return;
    const incoming = Array.from(fileList);
    const current = entries();
    if (current.length + incoming.length > MAX_FILES_PER_BATCH) {
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
    if (activeId() === null && added.length > 0) {
      setActiveId(added[0]!.id);
    }
  };

  const removeEntry = (id: string) => {
    setEntries((prev) => {
      const target = prev.find((e) => e.id === id);
      if (target) URL.revokeObjectURL(target.previewUrl);
      return prev.filter((e) => e.id !== id);
    });
    if (activeId() === id) {
      setActiveId(entries()[0]?.id ?? null);
    }
  };

  const updateComment = (id: string, comment: string) => {
    setEntries((prev) =>
      prev.map((e) => (e.id === id ? { ...e, comment } : e)),
    );
  };

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
    for (const entry of entries()) {
      formData.append("files", entry.file);
    }
    formData.append("meta", JSON.stringify(meta));
    formData.append("context", "photography");
    props.onSubmit(formData);
  };

  onMount(() => {
    if (!mapDiv) return;
    map = L.map(mapDiv).setView([20, 0], 2);
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
      setLocationForActive(result.location.y, result.location.x);
    }) as unknown as L.LeafletEventHandlerFn);

    map.on("click", (e) => {
      setLocationForActive(e.latlng.lat, e.latlng.lng);
    });
  });

  // Keep the marker in sync with the active row's location.
  createEffect(() => {
    const id = activeId();
    const active = entries().find((e) => e.id === id);
    if (!map) return;
    if (active && active.lat !== null && active.lon !== null) {
      if (marker) {
        marker.setLatLng([active.lat, active.lon]);
      } else {
        marker = L.marker([active.lat, active.lon], { icon: emojiIcon }).addTo(map);
      }
    } else if (marker) {
      marker.remove();
      marker = null;
    }
  });

  onCleanup(() => {
    for (const e of entries()) URL.revokeObjectURL(e.previewUrl);
    map?.remove();
    map = null;
  });

  const activeName = createMemo(() => {
    const active = entries().find((e) => e.id === activeId());
    return active?.file.name ?? "";
  });

  return (
    <form onSubmit={handleSubmit} class="flex flex-col gap-4">
      <h2 class={pageStyles.sectionTitle}>{t("photos.batch_upload_title")}</h2>

      <div>
        <label class={`block text-sm font-medium ${pageStyles.muted} mb-1`}>
          {t("photos.select_images")}
        </label>
        <input
          type="file"
          accept="image/*"
          multiple
          onChange={(e) => {
            handleFiles(e.currentTarget.files);
            e.currentTarget.value = "";
          }}
          class="block w-full text-sm text-slate-500 file:mr-4 file:py-2 file:px-4 file:rounded-full file:border-0 file:text-sm file:font-semibold file:bg-slate-100 file:text-slate-700 hover:file:bg-slate-200 dark:file:bg-slate-800 dark:file:text-slate-200"
        />
        <Show when={entries().length > 0}>
          <p class={`mt-1 text-xs ${pageStyles.muted}`}>
            {tx("photos.batch_count_selected", { count: entries().length })}
          </p>
        </Show>
      </div>

      <Show when={err()}>
        <p class={pageStyles.alertError}>{err()}</p>
      </Show>

      <Show when={entries().length > 0}>
        <div>
          <label class={`block text-sm font-medium ${pageStyles.muted} mb-1`}>
            {t("photos.location")}
          </label>
          <Show when={activeName()}>
            <p class="text-xs text-emerald-600 mb-1">
              {tx("photos.batch_active_row", { name: activeName() })}
            </p>
          </Show>
          <div ref={(el) => (mapDiv = el)} class="map-container" />
        </div>

        <div class="flex flex-col gap-3 max-h-80 overflow-y-auto pr-1">
          <For each={entries()}>
            {(entry) => (
              <div
                class={`flex gap-3 p-2 rounded-lg border ${
                  entry.id === activeId()
                    ? "border-emerald-400 dark:border-emerald-600"
                    : "border-slate-200 dark:border-slate-700"
                }`}
              >
                <img
                  src={entry.previewUrl}
                  alt={entry.file.name}
                  class="w-20 h-20 object-cover rounded shrink-0"
                />
                <div class="flex flex-col gap-1 grow min-w-0">
                  <p class="text-xs font-medium truncate" title={entry.file.name}>
                    {entry.file.name}
                  </p>
                  <textarea
                    value={entry.comment}
                    onInput={(e) => updateComment(entry.id, e.currentTarget.value)}
                    rows={2}
                    class={pageStyles.textarea}
                    placeholder={t("photos.comment_placeholder")}
                  />
                  <div class="flex items-center justify-between gap-2">
                    <span class="text-xs text-slate-500 dark:text-slate-400">
                      <Show
                        when={entry.lat !== null && entry.lon !== null}
                        fallback={t("photos.select_location")}
                      >
                        {tx("photos.selected", {
                          lat: entry.lat?.toFixed(5) ?? "",
                          lon: entry.lon?.toFixed(5) ?? "",
                        })}
                      </Show>
                    </span>
                    <div class="flex gap-2 shrink-0">
                      <button
                        type="button"
                        class={pageStyles.buttonGhost}
                        onClick={() => setActiveId(entry.id)}
                      >
                        {t("photos.batch_set_location")}
                      </button>
                      <button
                        type="button"
                        class={pageStyles.buttonGhost}
                        onClick={() => removeEntry(entry.id)}
                      >
                        {t("photos.batch_remove")}
                      </button>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </For>
        </div>
      </Show>

      <Show when={props.submitting}>
        <div class="w-full mt-2">
          <div class="w-full bg-slate-200 dark:bg-slate-700 rounded h-2 overflow-hidden">
            <div
              class="bg-slate-900 dark:bg-slate-100 h-2 transition-all duration-150"
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
