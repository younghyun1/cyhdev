import { createRouter } from "@solidjs/router";
import { beforeAll, describe, expect, it } from "vitest";

const storage = new Map<string, string>();
Object.defineProperty(window, "localStorage", { configurable: true, value: {
  getItem: (key: string) => storage.get(key) ?? null,
  setItem: (key: string, value: string) => storage.set(key, value),
  removeItem: (key: string) => storage.delete(key),
} });

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
    ["/forum", "/forum"],
    ["/forum/new", "/forum/new"],
    ["/forum/notifications", "/forum/notifications"],
    ["/forum/0198f4d0-aaaa-7000-8000-000000000001", "/forum/:topic_id"],
    ["/photographs/42", "/photographs/:photograph_id"],
    ["/admin/authorization", "/admin/authorization"],
    ["/admin/operations", "/admin/operations"],
  ])("matches %s with the expected route", (url, pattern) => {
    expect(Router.match(url).at(-1)?.pattern).toBe(pattern);
  });

  it("uses the named wildcard route for unknown URLs", () => {
    expect(Router.match("/missing/page").at(-1)?.pattern).toBe("/*404");
  });
});
