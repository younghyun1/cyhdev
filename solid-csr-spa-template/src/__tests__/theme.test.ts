import { describe, it, expect, beforeEach } from "vitest";

// Mock localStorage before importing the module
const storage: Record<string, string> = {};
Object.defineProperty(window, "localStorage", { configurable: true, value: {
  getItem: (key: string) => storage[key] ?? null,
  setItem: (key: string, value: string) => {
    storage[key] = value;
  },
  removeItem: (key: string) => {
    delete storage[key];
  },
} });

describe("theme state", () => {
  beforeEach(() => {
    for (const k of Object.keys(storage)) delete storage[k];
  });

  it("toggleTheme alternates between light and dark", async () => {
    const { theme, toggleTheme } = await import("../state/theme");
    const initial = theme();
    toggleTheme();
    expect(theme()).toBe(initial === "dark" ? "light" : "dark");
    toggleTheme();
    expect(theme()).toBe(initial);
  });

  it("toggleTheme persists to localStorage", async () => {
    const { toggleTheme } = await import("../state/theme");
    toggleTheme();
    expect(storage["theme"]).toBeDefined();
  });
});
