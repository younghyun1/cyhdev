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
    ["/about/experience", "/about/:page"],
    ["/about/background", "/about/:page"],
    ["/about-blog", "/about-blog"],
    ["/about-blog/performance", "/about-blog/:page"],
    ["/verify-email", "/verify-email"],
    ["/blog", "/blog"],
    ["/blog/example-post", "/blog/:post_id"],
    ["/forum", "/forum"],
    ["/forum/new", "/forum/new"],
    ["/forum/notifications", "/forum/notifications"],
    ["/forum/0198f4d0-aaaa-7000-8000-000000000001", "/forum/:topic_id"],
    ["/photographs/42", "/photographs/:photograph_id"],
    ["/eu5-locations-db", "/eu5-locations-db"],
    ["/admin/authorization", "/admin/authorization"],
    ["/admin/operations", "/admin/operations"],
    ["/admin/minecraft", "/admin/minecraft"],
    ["/projects", "/projects"],
    ["/projects/kcna-archive", "/projects/kcna-archive"],
    ["/projects/kcna-archive/documents/0198f4d0-aaaa-7000-8000-000000000001", "/projects/kcna-archive/documents/:documentId"],
    ["/projects/kcna-archive/trends", "/projects/kcna-archive/trends"],
    ["/projects/kcna-archive/status", "/projects/kcna-archive/status"],
  ])("matches %s with the expected route", (url, pattern) => {
    expect(Router.match(url).at(-1)?.pattern).toBe(pattern);
  });

  it("uses the named wildcard route for unknown URLs", () => {
    expect(Router.match("/missing/page").at(-1)?.pattern).toBe("/*404");
  });

  it.each([
    "/projects/kcna-archive/unknown",
    "/projects/kcna-archive/documents",
    "/projects/kcna-archive/trends/extra",
    "/projects/other",
  ])("sends the unknown archive path %s to the site 404", (url) => {
    expect(Router.match(url).at(-1)?.pattern).toBe("/*404");
  });

  it.each(["/about/finances", "/about-blog/secrets", "/about/experience/extra"])(
    "rejects unknown About subpage %s",
    (url) => {
      expect(Router.match(url).at(-1)?.pattern).toBe("/*404");
    },
  );
});
