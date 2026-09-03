import { createEffect, onCleanup, onSettled } from "solid-js";
import L from "leaflet";
import "leaflet/dist/leaflet.css";

type PhotographMapProps = {
  readonly lat: number;
  readonly lon: number;
};

export default function PhotographMap(props: PhotographMapProps) {
  let mapDiv: HTMLDivElement | undefined;
  let map: L.Map | null = null;
  let marker: L.Marker | null = null;
  let resizeObserver: ResizeObserver | null = null;

  const emojiIcon = L.divIcon({
    className: "emoji-marker",
    html: "📍",
    iconSize: [30, 30],
    iconAnchor: [15, 30],
  });

  const invalidate = () => map?.invalidateSize({ animate: false });

  onSettled(() => {
    if (!mapDiv) return;
    map = L.map(mapDiv).setView([props.lat, props.lon], 13);
    L.tileLayer("https://tile.openstreetmap.org/{z}/{x}/{y}.png", {
      attribution:
        '&copy; <a href="https://www.openstreetmap.org/copyright">OpenStreetMap</a> contributors',
    }).addTo(map);
    marker = L.marker([props.lat, props.lon], { icon: emojiIcon }).addTo(map);
    resizeObserver =
      typeof ResizeObserver === "undefined"
        ? null
        : new ResizeObserver(invalidate);
    resizeObserver?.observe(mapDiv);
    window.visualViewport?.addEventListener("resize", invalidate);
  });

  onCleanup(() => {
    resizeObserver?.disconnect();
    window.visualViewport?.removeEventListener("resize", invalidate);
    map?.remove();
    map = null;
    marker = null;
  });

  createEffect(
    () => [props.lat, props.lon] as const,
    ([lat, lon]) => {
      if (!map || !marker) return;
      map.setView([lat, lon], 13);
      marker.setLatLng([lat, lon]);
    },
  );

  return <div ref={(element) => (mapDiv = element)} class="map-container" />;
}
