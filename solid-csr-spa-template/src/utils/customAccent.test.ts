import { describe, expect, it } from "vitest";
import {
  contrastRatio,
  deriveCustomAccent,
  formatRgb,
  parseRgbInput,
  serializeCustomAccent,
} from "./customAccent";
import { storedCustomAccentSource } from "../state/accent";

const BACKGROUNDS = {
  light: ["#f6f1e8", "#fdfaf4"],
  dark: ["#000000", "#141210"],
} as const;

describe("parseRgbInput", () => {
  it.each([
    ["255, 120, 40", "#ff7828"],
    ["255 120 40", "#ff7828"],
    ["255,120,40", "#ff7828"],
    ["rgb(255, 120, 40)", "#ff7828"],
    ["RGB(255 120 40)", "#ff7828"],
    ["  0 0 0  ", "#000000"],
    ["#FF7828", "#ff7828"],
    ["ff7828", "#ff7828"],
    ["#f72", "#ff7722"],
  ])("reads %s", (input, expected) => {
    expect(parseRgbInput(input)).toBe(expected);
  });

  it.each(["", "nope", "256, 0, 0", "1, 2", "1, 2, 3, 4", "#ff78", "f72", "-1, 0, 0", "1.5, 2, 3"])(
    "rejects %j",
    (input) => {
      expect(parseRgbInput(input)).toBeNull();
    },
  );

  it("round-trips through the displayed form", () => {
    expect(parseRgbInput(formatRgb("#0a7f3c"))).toBe("#0a7f3c");
  });
});

describe("deriveCustomAccent", () => {
  it("keeps a color that already reads well in that mode", () => {
    expect(deriveCustomAccent("#1d4ed8").light.accent).toBe("#1d4ed8");
    expect(deriveCustomAccent("#fbbf24").dark.accent).toBe("#fbbf24");
  });

  it.each(["#ffff00", "#ffffff", "#000000", "#808080", "#00ff00", "#0000ff", "#ff0000", "#b45309"])(
    "keeps %s readable on every background in both modes",
    (source) => {
      const derived = deriveCustomAccent(source);
      for (const mode of ["light", "dark"] as const) {
        const { accent, soft } = derived[mode];
        for (const background of [...BACKGROUNDS[mode], soft]) {
          expect(contrastRatio(accent, background)).toBeGreaterThanOrEqual(4.5);
        }
      }
    },
  );

  it("moves an unreadable color only as far as it needs", () => {
    const { accent } = deriveCustomAccent("#ffff00").light;
    expect(accent).not.toBe("#ffff00");
    expect(contrastRatio(accent, BACKGROUNDS.light[0])).toBeLessThan(6);
  });
});

describe("stored custom accents", () => {
  it("round-trips the source color", () => {
    expect(storedCustomAccentSource(serializeCustomAccent("#ff7828"))).toBe("#ff7828");
  });

  it.each([
    null,
    "",
    "#ff7828",
    "--custom-accent-source:#ff7828;",
    "--custom-accent-source:red;--custom-accent-light:#000000;--custom-accent-strong-light:#000000;--custom-accent-soft-light:#000000;--custom-accent-dark:#000000;--custom-accent-strong-dark:#000000;--custom-accent-soft-dark:#000000;",
    `${serializeCustomAccent("#ff7828")}position:fixed;`,
  ])("rejects %j", (stored) => {
    expect(storedCustomAccentSource(stored)).toBeNull();
  });
});
