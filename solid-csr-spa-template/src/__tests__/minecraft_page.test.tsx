import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import Minecraft from "../pages/minecraft";
import { setLocaleSignal, setTexts } from "../state/i18n";

describe("Minecraft map page", () => {
  beforeEach(() => {
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("frames the map in an opaque-origin sandbox", () => {
    const result = render(() => <Minecraft />);
    const frame = result.container.querySelector("iframe");
    const sandbox = frame?.getAttribute("sandbox") ?? "";

    expect(frame?.getAttribute("src")).toBe("/minecraft/map/");
    expect(sandbox.split(" ")).toEqual([
      "allow-scripts",
      "allow-popups",
      "allow-popups-to-escape-sandbox",
    ]);
    expect(frame?.getAttribute("allow")).toBe("clipboard-write *");
  });
});
