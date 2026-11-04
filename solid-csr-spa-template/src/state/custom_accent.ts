import {
  customAccentProperties,
  parseRgbInput,
  serializeCustomAccent,
} from "../utils/customAccent";
import { commitCustomAccent, customColor } from "./accent";

const FALLBACK_COLOR = "#b45309";

export function setCustomAccent(color: string): void {
  const root = document.documentElement;
  for (const [name, value] of customAccentProperties(color)) {
    root.style.setProperty(name, value);
  }
  commitCustomAccent(color, serializeCustomAccent(color));
}

/** The last custom color, or at first the accent showing now, so nothing jumps. */
export function selectCustomAccent(): void {
  const showing = parseRgbInput(
    getComputedStyle(document.documentElement).getPropertyValue("--accent"),
  );
  setCustomAccent(customColor() ?? showing ?? FALLBACK_COLOR);
}
