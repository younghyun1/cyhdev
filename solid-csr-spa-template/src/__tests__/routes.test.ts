import { createRouter } from "@solidjs/router";
import { beforeAll, describe, expect, it, vi } from "vitest";

const storage = new Map<string, string>();
vi.stubGlobal("localStorage", {
  getItem: (key: string) => storage.get(key) ?? null,
  setItem: (key: string, value: string) => storage.set(key, value),
  removeItem: (key: string) => storage.delete(key),
});

let Router: ReturnType<typeof createRouter>;

beforeAll(async () => {
  const { routes } = await import("../routes");
  Router = createRouter({ routes });
});

describe("application routes", () => {
  it.each([
    ["/", ""],
    ["/about", "/about"],
    ["/verify-email", "/verify-email"],
    ["/blog", "/blog"],
    ["/blog/example-post", "/blog/:post_id"],
    ["/photographs/42", "/photographs/:photograph_id"],
  ])("matches %s with the expected route", (url, pattern) => {
    expect(Router.match(url).at(-1)?.pattern).toBe(pattern);
  });

  it("uses the named wildcard route for unknown URLs", () => {
    expect(Router.match("/missing/page").at(-1)?.pattern).toBe("/*404");
  });
});
