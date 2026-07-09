import {
  createSignal,
  createEffect,
  onCleanup,
  For,
  Show,
  onMount,
  createMemo,
  untrack,
} from "solid-js";
import { useNavigate, useLocation } from "@solidjs/router";
import type { RouteSectionProps } from "@solidjs/router";
import { photographyApi } from "../services/all_api";
import { isSuperuser } from "../state/auth";
import { pageStyles } from "../styles/pageStyles";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

import type {
  PhotographItem,
  GetPhotographsResponse,
} from "../dtos/responses/photography";
import { t, tx, locale } from "../state/i18n";
import BatchUploadFields from "../components/photographs/BatchUploadFields";
import ProcessingModal from "../components/photographs/ProcessingModal";
import PhotographSocial from "../components/photographs/PhotographSocial";
import {
  trackFromUpload,
  setBatchCompletionHandler,
  activeBatchCount,
} from "../state/photo_batches";

// --- Styles ---
const styles = `
/* Flex Masonry Layout */
.masonry-grid {
  display: flex;
  width: 100%;
  max-width: 1600px;
  gap: 1rem;
  padding: 1rem;
}
.masonry-column {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  flex: 1;
  min-width: 0; /* Prevents flex items from overflowing */
}

.photo-card {
  border-radius: 0.5rem;
  overflow: hidden;
  cursor: pointer;
  position: relative;
  transition: transform 0.2s, box-shadow 0.2s;
  background-color: var(--surface-2);
  width: 100%;
}
.photo-card:hover {
  transform: scale(1.02);
  z-index: 10;
  box-shadow: 0 10px 15px -3px rgba(0, 0, 0, 0.1);
}
.photo-card img {
  width: 100%;
  height: auto;
  display: block;
}
/* Small view/vote summary rendered below the image, outside the picture. */
.photo-meta {
  display: flex;
  align-items: center;
  gap: 0.85rem;
  padding: 0.4rem 0.6rem;
  font-size: 0.72rem;
  line-height: 1;
  color: var(--ink-muted);
}
.photo-meta .pm-item {
  display: inline-flex;
  align-items: center;
  gap: 0.25rem;
}
/* Month-year section header above each masonry block. */
.photo-section-title {
  width: 100%;
  max-width: 1600px;
  margin: 0 auto;
  padding: 1.25rem 1.75rem 0;
  font-size: 1.05rem;
  font-weight: 600;
  letter-spacing: -0.01em;
  color: var(--ink);
}

/* Modals */
.modal-overlay {
  position: fixed;
  inset: 0;
  background-color: rgba(0, 0, 0, 0.85);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 50;
  padding: 1rem;
}
.modal-content {
  background-color: var(--surface);
  border-radius: 0.5rem;
  max-width: 90vw;
  max-height: 90vh;
  overflow: auto;
  position: relative;
  display: flex;
  flex-direction: column;
}
.upload-modal {
  width: 700px;
  max-width: 100%;
}
.details-modal {
  max-width: 95vw;
  width: 100%;
  height: 90vh;
  flex-direction: row;
  overflow: hidden;
}
@media (max-width: 768px) {
  .details-modal {
    flex-direction: column;
    overflow-y: auto;
  }
  .details-image-container {
    width: 100%;
    height: 50vh;
  }
  .details-info {
    width: 100%;
    padding: 1rem;
  }
}
.details-image-container {
  flex: 3;
  background: black;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  position: relative;
}
.details-image-container {
  flex: 3;
  background: black;
  display: flex;
  align-items: center;
  justify-content: center;
  min-height: 400px;
  position: relative;
}
.nav-btn {
  opacity: 0;
  transition: opacity 0.3s ease-in-out;
}
.details-image-container:hover .nav-btn {
  opacity: 1;
}
@media (max-width: 768px) {
  .nav-btn {
    opacity: 1;
  }
}
.details-image-container img {
  max-width: 100%;
  max-height: 100%;
  object-fit: contain;
}
.details-info {
  flex: 1;
  padding: 2rem;
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
  overflow-y: auto;
  min-width: 300px;
}
.map-container {
  height: 300px;
  width: 100%;
  border-radius: 0.5rem;
  margin-top: 0.5rem;
  z-index: 1;
}
/* Leaflet Geosearch Customization */
.leaflet-control-geosearch form {
  background: var(--surface);
  border-radius: 4px;
  padding: 2px;
}
.leaflet-control-geosearch input {
  color: var(--ink);
}
.emoji-marker {
  font-size: 2rem;
  line-height: 1.2;
  text-align: center;
  transform: translateY(-10%);
}
.processing-modal {
  width: 720px;
  max-width: 100%;
}
`;

export default function Photographs(props: RouteSectionProps) {
  // State
  const [photos, setPhotos] = createSignal<PhotographItem[]>([]);
  const [page, setPage] = createSignal(1);
  const [loading, setLoading] = createSignal(false);
  const [hasMore, setHasMore] = createSignal(true);
  const [error, setError] = createSignal<string | null>(null);

  // Layout State for Masonry LTR
  const [numColumns, setNumColumns] = createSignal(1);

  // Modals
  const [selectedPhoto, setSelectedPhoto] = createSignal<PhotographItem | null>(
    null,
  );
  const [showUpload, setShowUpload] = createSignal(false);

  // Selection Mode State
  const [isSelectionMode, setIsSelectionMode] = createSignal(false);
  const [selectedForDeletion, setSelectedForDeletion] = createSignal<
    Set<string>
  >(new Set());

  // Upload Form State (batch)
  const [uploading, setUploading] = createSignal(false);
  const [uploadProgress, setUploadProgress] = createSignal(0);
  const [showProcessing, setShowProcessing] = createSignal(false);

  // --- URL-synced detail modal ---
  // The detail view is /photographs/:photograph_id rendered as a modal over the
  // (persistent) gallery. The id is read from the location so the gallery is not
  // remounted; opening/closing/navigating is just navigation.
  const navigate = useNavigate();
  const location = useLocation();
  const detailId = createMemo(() => {
    const m = location.pathname.match(/^\/photographs\/([^/]+)\/?$/);
    return m?.[1] ?? null;
  });
  const openPhoto = (id: string) => navigate(`/photographs/${id}`);
  const closePhoto = () => navigate("/photographs");

  // Resolve the route id to a photo: prefer the already-loaded list (so prev/next
  // is instant), else fetch the detail for a cold deep-link. Tracks detailId only
  // (photos read untracked) to avoid refetching on every infinite-scroll append.
  createEffect(() => {
    const id = detailId();
    if (!id) {
      setSelectedPhoto(null);
      return;
    }
    const inList = untrack(() =>
      photos().find((p) => p.photograph_id === id),
    );
    if (inList) {
      setSelectedPhoto(inList);
      return;
    }
    setSelectedPhoto(null);
    let cancelled = false;
    photographyApi
      .getPhotographDetail(id)
      .then((resp) => {
        if (!cancelled) setSelectedPhoto(resp.data.photograph);
      })
      .catch((err: unknown) => {
        console.error("Failed to load photograph detail:", err);
        if (!cancelled) navigate("/photographs", { replace: true });
      });
    onCleanup(() => {
      cancelled = true;
    });
  });

  // --- Layout Logic: month-year segments, each a masonry block ---
  const photoDate = (p: PhotographItem) =>
    new Date(p.photograph_shot_at ?? p.photograph_created_at);
  // Sortable key (YYYY-MM, month 0-based but consistent, so string sort works).
  const monthKey = (p: PhotographItem) => {
    const d = photoDate(p);
    return `${d.getFullYear()}-${String(d.getMonth()).padStart(2, "0")}`;
  };
  const monthLabel = (p: PhotographItem) =>
    photoDate(p).toLocaleDateString(locale(), {
      year: "numeric",
      month: "long",
    });

  interface PhotoSegment {
    key: string;
    label: string;
    photos: PhotographItem[];
  }

  const segments = createMemo<PhotoSegment[]>(() => {
    const groups = new Map<string, PhotoSegment>();
    for (const p of photos()) {
      const key = monthKey(p);
      let g = groups.get(key);
      if (!g) {
        g = { key, label: monthLabel(p), photos: [] };
        groups.set(key, g);
      }
      g.photos.push(p);
    }
    // Newest month first. Photos within a segment keep fetch order (shot_at desc).
    return Array.from(groups.values()).sort((a, b) =>
      a.key < b.key ? 1 : a.key > b.key ? -1 : 0,
    );
  });

  // Distribute a segment's photos LTR across the responsive column count.
  const columnsFor = (list: PhotographItem[]) => {
    const n = numColumns();
    const cols = Array.from({ length: n }, () => [] as PhotographItem[]);
    list.forEach((photo, i) => {
      cols[i % n]!.push(photo);
    });
    return cols;
  };

  const netVotes = (p: PhotographItem) =>
    p.photograph_total_upvotes - p.photograph_total_downvotes;

  const handleDelete = async () => {
    const ids = Array.from(selectedForDeletion());
    if (ids.length === 0) return;
    if (
      !confirm(
        tx("photos.delete_confirm", { count: ids.length }),
      )
    )
      return;

    try {
      setLoading(true);
      await photographyApi.deletePhotographs(ids);
      setPhotos((prev) =>
        prev.filter((p) => !selectedForDeletion().has(p.photograph_id)),
      );
      setIsSelectionMode(false);
      setSelectedForDeletion(new Set<string>());
    } catch (e) {
      alert(tx("photos.delete_failed", { error: String(e) }));
    } finally {
      setLoading(false);
    }
  };

  const toggleSelection = (id: string) => {
    setSelectedForDeletion((prev) => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  // Calculate columns based on window width
  onMount(() => {
    const updateColumns = () => {
      const w = window.innerWidth;
      if (w >= 1280) setNumColumns(4);
      else if (w >= 1024) setNumColumns(3);
      else if (w >= 640) setNumColumns(2);
      else setNumColumns(1);
    };

    updateColumns();
    window.addEventListener("resize", updateColumns);
    onCleanup(() => window.removeEventListener("resize", updateColumns));
  });

  // Load photos
  const fetchPhotos = async () => {
    if (loading() || !hasMore()) return;
    setLoading(true);
    try {
      const resp = await photographyApi.getPhotographs(page(), 24);
      const data = resp.data as GetPhotographsResponse;

      if (data.items.length === 0) {
        setHasMore(false);
      } else {
        setPhotos((prev) => [...prev, ...data.items]);
        setPage((p) => p + 1);
        setHasMore(data.pagination.has_next);
      }
    } catch (err: unknown) {
      console.error("Failed to fetch photos:", err);
      setError(t("photos.load_failed"));
    } finally {
      setLoading(false);
    }
  };

  // Initial load
  let sentinelEl: HTMLElement | null = null;
  onMount(() => {
    fetchPhotos();
    const observer = new IntersectionObserver(
      (entries) => {
        if (entries[0]?.isIntersecting && hasMore() && !loading()) {
          fetchPhotos();
        }
      },
      { threshold: 0.5 },
    );

    sentinelEl = document.getElementById("scroll-sentinel");
    if (sentinelEl) observer.observe(sentinelEl);

    onCleanup(() => observer.disconnect());
  });

  // Re-fetch when a page finishes loading but the sentinel is still on-screen
  // (IntersectionObserver only fires on transitions, so a short first page would otherwise stall).
  createEffect(() => {
    // track reactivity: re-run after each fetch settles and after the list grows
    photos();
    if (loading() || !hasMore() || !sentinelEl) return;
    const r = sentinelEl.getBoundingClientRect();
    const visible = r.top < window.innerHeight && r.bottom > 0;
    if (visible) fetchPhotos();
  });

  // Handle batch upload: fire the request, start tracking, open Processing.
  const handleBatchUpload = async (formData: FormData) => {
    setUploading(true);
    setUploadProgress(0);
    try {
      const resp = await photographyApi.batchUpload(formData, {
        onUploadProgress: (percent) => {
          setUploadProgress((prev) => (percent > prev ? percent : prev));
        },
      });
      setUploadProgress(100);
      trackFromUpload(resp.data);
      setShowUpload(false);
      setShowProcessing(true);
    } catch (err: unknown) {
      console.error("Batch upload failed:", err);
      alert(t("photos.upload_failed"));
    } finally {
      setUploading(false);
      setUploadProgress(0);
    }
  };

  // When any tracked batch finishes processing, reload the grid from page 1.
  onMount(() => {
    setBatchCompletionHandler(() => {
      setPhotos([]);
      setPage(1);
      setHasMore(true);
      fetchPhotos();
    });
  });
  onCleanup(() => setBatchCompletionHandler(null));

  // State for external map links popup
  const [showMapLinks, setShowMapLinks] = createSignal(false);

  // Map Component for Details - uses key prop to force remount on photo change
  const DetailsMap = (props: { lat: number; lon: number }) => {
    let mapDiv: HTMLDivElement | undefined;
    let map: L.Map | null = null;
    let marker: L.Marker | null = null;

    const emojiIcon = L.divIcon({
      className: "emoji-marker",
      html: "📍",
      iconSize: [30, 30],
      iconAnchor: [15, 30],
    });

    onMount(() => {
      map = L.map(mapDiv!).setView([props.lat, props.lon], 13);
      L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
        attribution:
          '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
      }).addTo(map);

      marker = L.marker([props.lat, props.lon], { icon: emojiIcon }).addTo(map);
    });

    // React to prop changes
    createEffect(() => {
      const lat = props.lat;
      const lon = props.lon;
      if (map && marker) {
        map.setView([lat, lon], 13);
        marker.setLatLng([lat, lon]);
      }
    });

    onCleanup(() => {
      map?.remove();
    });

    return <div ref={(el) => (mapDiv = el)} class="map-container" />;
  };

  // External map link generators
  const getGoogleMapsUrl = (lat: number, lon: number) =>
    `https://www.google.com/maps?q=${lat},${lon}`;
  const getGoogleEarthUrl = (lat: number, lon: number) =>
    `https://earth.google.com/web/@${lat},${lon},0a,1000d,35y,0h,0t,0r`;
  const getOpenStreetMapUrl = (lat: number, lon: number) =>
    `https://www.openstreetmap.org/?mlat=${lat}&mlon=${lon}&zoom=15`;

  // Close map links popup when photo changes
  createEffect(() => {
    selectedPhoto(); // track changes
    setShowMapLinks(false);
  });

  // --- Navigation Logic ---
  // Neighbours are resolved from the loaded list and navigated to by URL (the
  // route effect then swaps the modal content). On a cold deep-link the photo is
  // not in the list (idx === -1), so prev/next are unavailable.
  const navigatePhoto = async (direction: "prev" | "next") => {
    const current = selectedPhoto();
    if (!current) return;

    const currentPhotos = photos();
    const idx = currentPhotos.findIndex(
      (p) => p.photograph_id === current.photograph_id,
    );
    if (idx === -1) return;

    if (direction === "prev" && idx > 0) {
      const prev = currentPhotos[idx - 1];
      if (prev) openPhoto(prev.photograph_id);
    } else if (direction === "next") {
      if (idx < currentPhotos.length - 1) {
        const next = currentPhotos[idx + 1];
        if (next) openPhoto(next.photograph_id);
      } else if (hasMore() && !loading()) {
        await fetchPhotos();
        const next = photos()[idx + 1];
        if (next) openPhoto(next.photograph_id);
      }
    }
  };

  createEffect(() => {
    // Only listen when a photo is selected
    if (!selectedPhoto()) return;

    const handleKeyDown = (e: KeyboardEvent) => {
      if (e.key === "Escape") {
        if (selectedPhoto()) closePhoto();
        else if (isSelectionMode()) {
          setIsSelectionMode(false);
          setSelectedForDeletion(new Set<string>());
        }
      }
      if (e.key === "ArrowLeft") navigatePhoto("prev");
      if (e.key === "ArrowRight") navigatePhoto("next");
    };

    window.addEventListener("keydown", handleKeyDown);
    onCleanup(() => window.removeEventListener("keydown", handleKeyDown));
  });

  return (
    <>
      <style>{styles}</style>
      <main class={pageStyles.page}>
        <div class="flex flex-col items-center w-full">
          {/* Header / Actions */}
          <div class="w-full max-w-400 px-6 py-6 flex flex-wrap gap-4 justify-between items-center">
            <h1 class={pageStyles.titleSm}>{t("page.photographs.title")}</h1>
            <Show when={isSuperuser()}>
              <button
                class={pageStyles.buttonPrimary}
                onClick={() => setShowUpload(true)}
              >
                {t("photos.upload_photo")}
              </button>
            </Show>

            <Show when={isSuperuser()}>
              <button
                class={pageStyles.buttonSecondary}
                onClick={() => setShowProcessing(true)}
              >
                {t("photos.processing")}
                <Show when={activeBatchCount() > 0}>
                  <span class="ml-2 inline-flex items-center justify-center rounded-full bg-accent text-paper text-xs font-bold h-5 min-w-5 px-1">
                    {activeBatchCount()}
                  </span>
                </Show>
              </button>
            </Show>

            <div class="flex gap-2 ml-4">
              <Show when={isSuperuser()}>
                <Show
                  when={isSelectionMode()}
                  fallback={
                    <button
                      class={pageStyles.buttonSecondary}
                      onClick={() => setIsSelectionMode(true)}
                    >
                      {t("photos.select")}
                    </button>
                  }
                >
                  <button
                    class={pageStyles.buttonDanger}
                    disabled={selectedForDeletion().size === 0 || loading()}
                    onClick={handleDelete}
                  >
                    {t("common.delete")} ({selectedForDeletion().size})
                  </button>
                  <button
                    class={pageStyles.buttonSecondary}
                    onClick={() => {
                      setIsSelectionMode(false);
                      setSelectedForDeletion(new Set<string>());
                    }}
                  >
                    {t("common.cancel")}
                  </button>
                </Show>
              </Show>
            </div>
          </div>

          {/* Error Message */}
          <Show when={error()}>
            <div class={`${pageStyles.alertError} w-full max-w-400 mb-4`}>
              {error()}
            </div>
          </Show>

          {/* Month-year segmented masonry */}
          <For each={segments()}>
            {(seg) => (
              <>
                <h2 class="photo-section-title">{seg.label}</h2>
                <div class="masonry-grid mx-auto">
                  <For each={columnsFor(seg.photos)}>
                    {(colPhotos) => (
                      <div class="masonry-column">
                        <For each={colPhotos}>
                          {(photo) => (
                            <div
                              class="photo-card"
                              onClick={() => {
                                if (isSelectionMode()) {
                                  toggleSelection(photo.photograph_id);
                                } else {
                                  openPhoto(photo.photograph_id);
                                }
                              }}
                              title={photo.photograph_comments}
                            >
                              <img
                                src={
                                  photo.photograph_thumbnail_link ||
                                  photo.photograph_link
                                }
                                alt={photo.photograph_comments}
                                loading="lazy"
                              />
                              <Show when={isSelectionMode()}>
                                <div
                                  class={`absolute inset-0 transition-all z-10 ${
                                    selectedForDeletion().has(
                                      photo.photograph_id,
                                    )
                                      ? "ring-4 ring-danger ring-inset bg-black/20"
                                      : "hover:bg-black/10"
                                  }`}
                                >
                                  <div
                                    class={`absolute top-2 right-2 w-6 h-6 rounded-full border-2 border-white flex items-center justify-center ${
                                      selectedForDeletion().has(
                                        photo.photograph_id,
                                      )
                                        ? "bg-danger"
                                        : "bg-black/40"
                                    }`}
                                  >
                                    <Show
                                      when={selectedForDeletion().has(
                                        photo.photograph_id,
                                      )}
                                    >
                                      <span class="text-white text-xs font-bold">
                                        ✓
                                      </span>
                                    </Show>
                                  </div>
                                </div>
                              </Show>
                              {/* View / vote summary, bottom-left, outside the pic */}
                              <div class="photo-meta">
                                <span
                                  class="pm-item"
                                  title={t("common.views")}
                                >
                                  <span aria-hidden="true">👁</span>
                                  {photo.photograph_view_count}
                                </span>
                                <span
                                  class="pm-item"
                                  title={t("blog.vote.upvote")}
                                >
                                  <span aria-hidden="true">▲</span>
                                  {netVotes(photo)}
                                </span>
                              </div>
                            </div>
                          )}
                        </For>
                      </div>
                    )}
                  </For>
                </div>
              </>
            )}
          </For>

          {/* Loading / Sentinel */}
          <div id="scroll-sentinel" class="h-10 w-full flex justify-center p-4">
            <Show when={loading()}>
              <span class={pageStyles.muted}>{t("photos.loading_more")}</span>
            </Show>
            <Show when={!hasMore() && photos().length > 0}>
              <span class={pageStyles.muted}>{t("photos.no_more")}</span>
            </Show>
          </div>
        </div>
      </main>

      {/* Param child route (/photographs/:photograph_id) renders nothing; it
          only keeps this page mounted while the detail modal is open. */}
      {props.children}


      {/* Upload Modal */}
      <Show when={showUpload()}>
        <div
          class="modal-overlay"
          onClick={(e) => {
            if (e.target === e.currentTarget) setShowUpload(false);
          }}
        >
          <div class="modal-content upload-modal p-6">
            <BatchUploadFields
              submitting={uploading()}
              progress={uploadProgress()}
              onSubmit={handleBatchUpload}
              onCancel={() => setShowUpload(false)}
            />
          </div>
        </div>
      </Show>

      <ProcessingModal
        show={showProcessing()}
        onClose={() => setShowProcessing(false)}
      />

      {/* Details Modal */}
      <Show when={selectedPhoto()}>
        <div
          class="modal-overlay"
          onClick={(e) => {
            if (e.target === e.currentTarget) closePhoto();
          }}
        >
          <div class="modal-content details-modal">
            <button
              onClick={() => closePhoto()}
              class="absolute top-4 right-4 z-10 p-2 bg-black/50 text-white rounded-full hover:bg-black/70"
            >
              ✕
            </button>
            <div class="details-image-container">
              {/* --- PREV BUTTON --- */}
              <Show
                when={
                  photos().findIndex(
                    (p) => p.photograph_id === selectedPhoto()?.photograph_id,
                  ) > 0
                }
              >
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    navigatePhoto("prev");
                  }}
                  // ADDED: nav-btn class
                  class="nav-btn absolute left-4 top-1/2 -translate-y-1/2 p-3 bg-black/50 hover:bg-black/70 text-white rounded-full z-20 backdrop-blur-sm transition-all hover:scale-110"
                  title={t("photos.previous")}
                >
                  <svg
                    xmlns="http://www.w3.org/2000/svg"
                    fill="none"
                    viewBox="0 0 24 24"
                    stroke-width="2"
                    stroke="currentColor"
                    class="w-6 h-6"
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      d="M15.75 19.5L8.25 12l7.5-7.5"
                    />
                  </svg>
                </button>
              </Show>

              <img
                src={selectedPhoto()!.photograph_link}
                alt={selectedPhoto()!.photograph_comments}
              />

              {/* --- NEXT BUTTON --- */}
              <Show
                when={
                  // CHANGED CONDITION: Show if not last element OR if server has more
                  photos().findIndex(
                    (p) => p.photograph_id === selectedPhoto()?.photograph_id,
                  ) <
                    photos().length - 1 || hasMore()
                }
              >
                <button
                  onClick={(e) => {
                    e.stopPropagation();
                    navigatePhoto("next");
                  }}
                  // ADDED: nav-btn class
                  class="nav-btn absolute right-4 top-1/2 -translate-y-1/2 p-3 bg-black/50 hover:bg-black/70 text-white rounded-full z-20 backdrop-blur-sm transition-all hover:scale-110"
                  title={t("photos.next")}
                >
                  {/* Optional: Show spinner if loading next page while hovering next button */}
                  <Show
                    when={
                      loading() &&
                      photos().findIndex(
                        (p) =>
                          p.photograph_id === selectedPhoto()?.photograph_id,
                      ) ===
                        photos().length - 1
                    }
                    fallback={
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="2"
                        stroke="currentColor"
                        class="w-6 h-6"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          d="M8.25 4.5l7.5 7.5-7.5 7.5"
                        />
                      </svg>
                    }
                  >
                    {/* Small Loading Spinner Icon */}
                    <svg
                      class="animate-spin w-6 h-6 text-white"
                      xmlns="http://www.w3.org/2000/svg"
                      fill="none"
                      viewBox="0 0 24 24"
                    >
                      <circle
                        class="opacity-25"
                        cx="12"
                        cy="12"
                        r="10"
                        stroke="currentColor"
                        stroke-width="4"
                      />
                      <path
                        class="opacity-75"
                        fill="currentColor"
                        d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z"
                      />
                    </svg>
                  </Show>
                </button>
              </Show>

              {/* Open original / View image */}
              <a
                href={selectedPhoto()!.photograph_link}
                target="_blank"
                rel="noopener noreferrer"
                // ADDED: nav-btn class (optional, if you want this to fade too)
                class="nav-btn absolute top-4 right-4 p-2 bg-black/40 hover:bg-black/60 text-white rounded-full backdrop-blur-sm transition-colors"
                title={t("photos.open_original")}
                aria-label={t("photos.open_original_aria")}
              >
                {/* External link icon */}
                <svg
                  xmlns="http://www.w3.org/2000/svg"
                  fill="none"
                  viewBox="0 0 24 24"
                  stroke-width="2"
                  stroke="currentColor"
                  class="w-6 h-6"
                >
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M13.5 6H18m0 0v4.5M18 6l-7.5 7.5"
                  />
                  <path
                    stroke-linecap="round"
                    stroke-linejoin="round"
                    d="M10.5 6H7.5A1.5 1.5 0 006 7.5v9A1.5 1.5 0 007.5 18h9a1.5 1.5 0 001.5-1.5V13.5"
                  />
                </svg>
              </a>
            </div>
            <div class="details-info bg-surface">
              <div>
                <h3 class="text-lg font-bold text-ink">
                  {t("photos.comments")}
                </h3>
                <p class="text-ink-muted mt-1 whitespace-pre-wrap">
                  {selectedPhoto()!.photograph_comments}
                </p>
              </div>

              <div>
                <h3 class="text-sm font-bold text-ink-muted uppercase tracking-wide">
                  {t("photos.taken_at")}
                </h3>
                <p class="text-ink">
                  {selectedPhoto()!.photograph_shot_at
                    ? new Date(
                        selectedPhoto()!.photograph_shot_at!,
                      ).toLocaleString()
                    : t("common.unknown_date")}
                </p>
              </div>

              <div>
                <h3 class="text-sm font-bold text-ink-muted uppercase tracking-wide">
                  {t("geo.coordinates")}
                </h3>
                <p class="font-mono text-sm text-ink">
                  {selectedPhoto()!.photograph_lat.toFixed(6)},{" "}
                  {selectedPhoto()!.photograph_lon.toFixed(6)}
                </p>
              </div>

              <div>
                <div class="flex items-center justify-between">
                  <h3 class="text-sm font-bold text-ink-muted uppercase tracking-wide">
                    {t("photos.location_map")}
                  </h3>
                  <div class="relative">
                    <button
                      onClick={() => setShowMapLinks(!showMapLinks())}
                      class="p-1.5 text-ink-muted hover:text-ink hover:bg-surface-2 rounded-sm transition-colors"
                      title={t("photos.open_external_map")}
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        fill="none"
                        viewBox="0 0 24 24"
                        stroke-width="1.5"
                        stroke="currentColor"
                        class="w-5 h-5"
                      >
                        <path
                          stroke-linecap="round"
                          stroke-linejoin="round"
                          d="M13.5 6H5.25A2.25 2.25 0 003 8.25v10.5A2.25 2.25 0 005.25 21h10.5A2.25 2.25 0 0018 18.75V10.5m-10.5 6L21 3m0 0h-5.25M21 3v5.25"
                        />
                      </svg>
                    </button>
                    <Show when={showMapLinks()}>
                      <div class="absolute right-0 top-full mt-1 bg-surface border border-line rounded-sm shadow-lg z-50 min-w-40 py-1">
                        <a
                          href={getGoogleMapsUrl(
                            selectedPhoto()!.photograph_lat,
                            selectedPhoto()!.photograph_lon,
                          )}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="flex items-center gap-2 px-3 py-2 text-sm text-ink hover:bg-surface-2 transition-colors"
                          onClick={() => setShowMapLinks(false)}
                        >
                          <span>Google Maps</span>
                        </a>
                        <a
                          href={getGoogleEarthUrl(
                            selectedPhoto()!.photograph_lat,
                            selectedPhoto()!.photograph_lon,
                          )}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="flex items-center gap-2 px-3 py-2 text-sm text-ink hover:bg-surface-2 transition-colors"
                          onClick={() => setShowMapLinks(false)}
                        >
                          <span>Google Earth</span>
                        </a>
                        <a
                          href={getOpenStreetMapUrl(
                            selectedPhoto()!.photograph_lat,
                            selectedPhoto()!.photograph_lon,
                          )}
                          target="_blank"
                          rel="noopener noreferrer"
                          class="flex items-center gap-2 px-3 py-2 text-sm text-ink hover:bg-surface-2 transition-colors"
                          onClick={() => setShowMapLinks(false)}
                        >
                          <span>OpenStreetMap</span>
                        </a>
                      </div>
                    </Show>
                  </div>
                </div>
                <div class="mt-2 h-50 rounded-sm overflow-hidden relative">
                  <DetailsMap
                    lat={selectedPhoto()!.photograph_lat}
                    lon={selectedPhoto()!.photograph_lon}
                  />
                </div>
              </div>

              <PhotographSocial
                photographId={selectedPhoto()!.photograph_id}
              />
            </div>
          </div>
        </div>
      </Show>
    </>
  );
}
