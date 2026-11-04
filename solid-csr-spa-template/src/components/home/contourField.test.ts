import { describe, expect, it } from "vitest";
import { createFrameProbe } from "./contourField";

const run = (intervalMs: number, callbacks: number): boolean => {
  const probe = createFrameProbe();
  let slow = false;
  for (let index = 0; index < callbacks; index += 1) {
    slow = probe(intervalMs) || slow;
  }
  return slow;
};

describe("contour field frame probe", () => {
  it("keeps animating at normal display rates", () => {
    expect(run(1000 / 60, 200)).toBe(false);
    expect(run(1000 / 30, 200)).toBe(false);
  });

  it("stops a device that cannot reach 20 frames per second", () => {
    expect(run(70, 200)).toBe(true);
  });

  it("ignores slow frames while the shader compiles", () => {
    const probe = createFrameProbe();
    const results = [probe(900), probe(900)];
    for (let index = 0; index < 100; index += 1) results.push(probe(16));
    expect(results).not.toContain(true);
  });
});
