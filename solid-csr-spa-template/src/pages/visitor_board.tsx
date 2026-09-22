import { createEffect, onSettled } from "solid-js";
import { visitorBoardApi } from "../services/all_api";
import { pageStyles } from "../styles/pageStyles";
import "leaflet/dist/leaflet.css";
import "../styles/visitor-board.css";
import L from "leaflet";
import { t, tx, locale, texts } from "../state/i18n";
import { visitorPopupText } from "../utils/statusMetadata";

const WORLD_BOUNDS = L.latLngBounds([-85.0511, -180], [85.0511, 180]);
const MARKER_EMOJI = "📍";

export default function VisitorBoard() {
  let mapDiv: HTMLDivElement | undefined;
  let map: L.Map | null = null;
  let markers: L.Marker[] = [];
  let markerCounts: number[] = [];
  let resizeObserver: ResizeObserver | null = null;

  onSettled(() => {
    let disposed = false;
    async function loadVisitorBoard() {
      try {
        const resp = await visitorBoardApi.getVisitorBoard();
        if (disposed || !mapDiv) return;
        const pairs = resp.data;
        const first = pairs[0];
        const initialLatLng: [number, number] =
          first ? [first[0][0] ?? 51.505, first[0][1] ?? -0.09] : [51.505, -0.09];

        // Remove old map if present
        if (map && map.remove) {
          map.remove();
        }
        map = L.map(mapDiv, {
          maxBounds: WORLD_BOUNDS,
          maxBoundsViscosity: 1.0,
        }).setView(initialLatLng, 3);

        L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
          attribution:
            '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
          bounds: WORLD_BOUNDS,
          noWrap: true,
        }).addTo(map);

        // Add all markers with a large 📍 emoji as the marker icon
        const emojiIcon = L.divIcon({
          className: "emoji-marker",
          html: MARKER_EMOJI,
          iconSize: [30, 30],
          iconAnchor: [15, 30],
          popupAnchor: [0, -30],
        });

        markerCounts = pairs.map((pair) => pair[1]);
        markers = pairs.map((pair) => {
          const [[lat, lng], count] = pair;
          const popup = document.createElement("span");
          popup.textContent = visitorPopupText(tx("visitor.popup", { count }));
          return L.marker([lat ?? 0, lng ?? 0], { icon: emojiIcon })
            .addTo(map!)
            .bindPopup(popup);
        });

        if (markers.length > 0) {
          markers[0]?.openPopup();
        }
        resizeObserver =
          typeof ResizeObserver === "undefined" || !mapDiv
            ? null
            : new ResizeObserver(() =>
                map?.invalidateSize({ animate: false }),
              );
        if (mapDiv) resizeObserver?.observe(mapDiv);
      } catch {
        if (disposed) return;
        if (map && map.remove) map.remove();
        map = null;
        if (mapDiv)
          mapDiv.textContent = t("visitor.load_failed");
      }
    }
    loadVisitorBoard();

    return () => {
      disposed = true;
      if (map && map.remove) {
        map.remove();
      }
      markers = [];
      markerCounts = [];
      resizeObserver?.disconnect();
      resizeObserver = null;
    };
  });

  // Refresh popup text reactively when the locale/text bundle changes.
  createEffect(
    () => [locale(), texts()] as const,
    () => {
      markers.forEach((marker, i) => {
        const popup = document.createElement("span");
        popup.textContent = visitorPopupText(tx("visitor.popup", { count: markerCounts[i] ?? 0 }));
        marker.setPopupContent(popup);
      });
    },
  );

  return (
    <main class={`${pageStyles.page} flex flex-col`}>
      <div class="visitor-board-center-outer">
        <div class="visitor-board-wrapper">
          <div ref={(el) => (mapDiv = el)} id="map" class="visitor-board-map" />
        </div>
      </div>
      <p class="pb-4 text-center text-xs text-ink-faint">
        {t("geo.attribution_prefix")}{" "}
        <a
          href="https://lite.ip2location.com"
          target="_blank"
          rel="noopener noreferrer"
          class="underline hover:text-ink-muted"
        >
          {t("geo.attribution_link")}
        </a>
        .
      </p>
    </main>
  );
}
