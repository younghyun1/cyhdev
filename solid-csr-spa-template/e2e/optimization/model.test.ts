import { describe, expect, it } from "vitest";
import { readFileSync } from "node:fs";
import { resolve } from "node:path";
import { currentRoutes, pagePath, routes } from "./inventory";
import { interpolate, localUrl, pointer, validate, type Campaign } from "./model";
import { operationMatches } from "./steps";
import { validateEnvironment } from "./isolation";

describe("live optimization inventory", () => {
  it("derives all current routes and effective access from the source tree", () => {
    const policy = JSON.parse(readFileSync(resolve(import.meta.dirname, "../../../tools/optimization/coverage.json"), "utf8")) as { pages: string[] };
    expect(currentRoutes().map(({ pattern }) => pattern).sort()).toEqual([...policy.pages].sort());
    expect(currentRoutes().find(({ pattern }) => pattern === "/admin/minecraft")?.actor).toBe("admin");
    expect(currentRoutes().find(({ pattern }) => pattern === "/blog/new")?.actor).toBe("member");
  });
  it("joins nested paths and preserves parent gallery access", () => {
    expect(routes(`defineRoutes([{path:"/items",component: withAuth(Page),children:[{path:"/",component:Index},{path:"/:id",component:Detail}]}]);`))
      .toEqual([{ pattern: "/items", actor: "member" }, { pattern: "/items/:id", actor: "member" }]);
  });
  it("requires actual entity fixtures and safely encodes page parameters", () => {
    expect(() => pagePath("/blog/:post_id", {})).toThrow();
    expect(pagePath("/users/:userName", { userName: "a/b" })).toBe("/users/a%2Fb");
    expect(pagePath("*404", {})).toBe("/optimization-unmatched-page");
  });
});

describe("disposable runtime admission", () => {
  const fixture = (): NodeJS.ProcessEnv => ({ CYHDEV_OPT_DISPOSABLE: "1", DB_URL: "postgresql://fixture:fixture@127.0.0.1/cyhdev_optimization_test",
    AWS_SES_SMTP_URL: "127.0.0.1", AWS_ENDPOINT_URL: "http://127.0.0.1:18500", OIDC_ISSUER_URL: "http://127.0.0.1:18501",
    AWS_IMAGE_UPLOAD_KEY: "optimization-fixture-key", AWS_IMAGE_UPLOAD_SECRET_KEY: "optimization-fixture-secret",
    AWS_SES_SMTP_USERNAME: "optimization-fixture-user", AWS_SES_SMTP_ACCESS_KEY: "optimization-fixture-password" });
  it("admits explicitly synthetic loopback services", () => {
    expect(() => validateEnvironment(fixture(), "/tmp/disposable-runtime")).not.toThrow();
  });
  it("rejects real databases, SMTP, object stores and OIDC", () => {
    for (const [key, value] of [["DB_URL", "postgresql://fixture:fixture@127.0.0.1/cyhdev"], ["DB_URL", "postgresql://fixture:fixture@db.example/cyhdev_optimization_test"],
      ["AWS_SES_SMTP_URL", "smtp.example"], ["AWS_ENDPOINT_URL", "https://s3.example"], ["OIDC_ISSUER_URL", "https://accounts.example"]]) {
      const env = fixture(); env[key!] = value;
      expect(() => validateEnvironment(env, "/tmp/disposable-runtime")).toThrow();
    }
  });
  it("rejects DB URL bypass, inherited real keys, and production socket paths", () => {
    for (const [key, value] of [["DB_HOST", "/run/postgresql"], ["AWS_IMAGE_UPLOAD_KEY", "real-key"], ["MINECRAFT_WORLD_SOCKET", "/run/world.sock"], ["SQUAREMAP_WEB_DIR", "/srv/map"]]) {
      const env = fixture(); env[key!] = value;
      expect(() => validateEnvironment(env, "/tmp/disposable-runtime")).toThrow();
    }
    const env = fixture(); delete env.CYHDEV_OPT_DISPOSABLE;
    expect(() => validateEnvironment(env, "/tmp/disposable-runtime")).toThrow();
  });
});
describe("live campaign protocol bounds", () => {
  it("rejects cross-origin paths and credentials", () => {
    expect(() => localUrl("https://127.0.0.1:18443", "https://other.example/api")).toThrow();
    expect(() => localUrl("https://127.0.0.1:18443", "//other.example/api")).toThrow();
    expect(localUrl("https://127.0.0.1:18443", "/api/items")).toBe("https://127.0.0.1:18443/api/items");
  });
  it("matches route segments rather than ambiguous prefixes", () => {
    expect(operationMatches("/api/items/{id}", "/api/items/fixture?q=a")).toBe(true);
    expect(operationMatches("/api/items/{id}", "/api/items/fixture/extra")).toBe(false);
    expect(operationMatches("/api/items/{id}", "/api/items/")).toBe(false);
  });
  it("resolves escaped JSON pointers and fails unresolved captures", () => {
    expect(pointer({ "a/b": { "~": 4 } }, "/a~1b/~0")).toBe(4);
    expect(() => pointer({ value: 1 }, "/missing")).toThrow();
    expect(interpolate("/items/${id}", { id: "fixture" })).toBe("/items/fixture");
    expect(() => interpolate("${MISSING_OPT_FIXTURE_VALUE}", {})).toThrow();
  });
  it("rejects remote backends and oversized workloads before any reset hook", () => {
    const campaign = JSON.parse(readFileSync(resolve(import.meta.dirname, "../../../tools/optimization/campaign.example.json"), "utf8")) as Campaign;
    expect(() => validate(campaign)).not.toThrow();
    campaign.base_url = "https://production.example:18443";
    expect(() => validate(campaign)).toThrow();
    campaign.base_url = "https://127.0.0.1:18443";
    campaign.workflows[0]!.repetitions = 101;
    expect(() => validate(campaign)).toThrow();
  });
});
