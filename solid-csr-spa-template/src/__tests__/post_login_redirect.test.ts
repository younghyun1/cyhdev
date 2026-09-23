import { afterEach, describe, expect, it } from "vitest";
import {
  consumePostLoginRedirect,
  rememberPostLoginRedirect,
  safeRedirectTarget,
} from "../services/api";

const ORIGIN = "https://cyhdev.example";

afterEach(() => sessionStorage.clear());

describe("post-login redirect validation", () => {
  it.each([
    ["/", "/"],
    ["/blog/7?tab=comments#reply-3", "/blog/7?tab=comments#reply-3"],
    ["/photographs/./abc", "/photographs/abc"],
    ["/forum/%5Cnot-a-host", "/forum/%5Cnot-a-host"],
  ])("keeps the same-origin path %s", (target, expected) => {
    expect(safeRedirectTarget(target, ORIGIN)).toBe(expected);
  });

  it.each([
    "//evil.example",
    "/\\evil.example",
    "/\\/evil.example",
    "/\t/evil.example",
    "/\n/evil.example",
    "/./\\evil.example",
    "\\\\evil.example",
    "https://evil.example/",
    `${ORIGIN}/blog`,
    "javascript:alert(1)",
    "blog",
    "",
    "/login",
    "/login?next=%2Fblog",
    "/login/",
  ])("rejects %j", (target) => {
    expect(safeRedirectTarget(target, ORIGIN)).toBeNull();
  });

  it("rejects missing values", () => {
    expect(safeRedirectTarget(null, ORIGIN)).toBeNull();
    expect(safeRedirectTarget(undefined, ORIGIN)).toBeNull();
  });

  it("stores only validated targets and clears them on read", () => {
    rememberPostLoginRedirect("/\\evil.example");
    expect(consumePostLoginRedirect()).toBeNull();

    rememberPostLoginRedirect("/forum/42?page=2");
    expect(consumePostLoginRedirect()).toBe("/forum/42?page=2");
    expect(consumePostLoginRedirect()).toBeNull();
  });

  it("revalidates a target written to storage by other code", () => {
    sessionStorage.setItem("post_login_redirect", "/\\evil.example");
    expect(consumePostLoginRedirect()).toBeNull();
    expect(sessionStorage.getItem("post_login_redirect")).toBeNull();
  });
});
