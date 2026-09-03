import { type Accessor, createSignal, onSettled } from "solid-js";

/** Reactive matchMedia wrapper for behavior that cannot be expressed in CSS. */
export function createMediaQuery(query: string): Accessor<boolean> {
  const initial =
    typeof window !== "undefined" && typeof window.matchMedia === "function"
      ? window.matchMedia(query).matches
      : false;
  const [matches, setMatches] = createSignal(initial);

  onSettled(() => {
    if (typeof window.matchMedia !== "function") return;
    const media = window.matchMedia(query);
    const update = (event?: MediaQueryListEvent) =>
      setMatches(event?.matches ?? media.matches);
    update();
    media.addEventListener("change", update);
    return () => media.removeEventListener("change", update);
  });

  return matches;
}
