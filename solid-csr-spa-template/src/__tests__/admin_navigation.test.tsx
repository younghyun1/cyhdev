import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import AdminNavigation, {
  ADMIN_NAVIGATION_LINKS,
} from "../components/AdminNavigation";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import { handleAdminForbiddenResponse } from "../services/api";
import { isSuperuser, setAuthenticated, setSuperuser } from "../state/auth";
import { setLocaleSignal, setTexts } from "../state/i18n";

const renderedHrefs = (container: HTMLElement): ReadonlyArray<string> =>
  Array.from(container.querySelectorAll("a"), (link) =>
    link.getAttribute("href"),
  ).filter((href): href is string => href !== null);

describe("administration navigation", () => {
  beforeEach(() => {
    setAuthenticated(false);
    setSuperuser(false);
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("keeps operations and OpenAPI in the privileged inventory", () => {
    const hrefs = ADMIN_NAVIGATION_LINKS.map((link) => link.href);
    expect(hrefs).toContain("/admin/operations");
    expect(hrefs).toContain("/swagger-ui");
  });

  it("drops stale admin controls after an authoritative admin denial", async () => {
    setSuperuser(true);
    await Promise.resolve();
    handleAdminForbiddenResponse("/api/admin/media-cleanup/unresolved");
    await Promise.resolve();
    expect(isSuperuser()).toBe(false);

    setSuperuser(true);
    await Promise.resolve();
    handleAdminForbiddenResponse("/api/administrator-profile");
    await Promise.resolve();
    expect(isSuperuser()).toBe(true);
  });

  it("stays absent until authentication confirms superuser access", () => {
    setAuthenticated(true);
    const result = render(() => (
      <AdminNavigation variant="desktop" isActive={() => false} />
    ));

    expect(renderedHrefs(result.container)).toEqual([]);

    setAuthenticated(false);
    setSuperuser(true);
    expect(renderedHrefs(result.container)).toEqual([]);
  });

  it.each(["desktop", "mobile"] as const)(
    "exposes every privileged surface in the %s menu",
    (variant) => {
      setAuthenticated(true);
      setSuperuser(true);

      const result = render(() => (
        <AdminNavigation variant={variant} isActive={() => false} />
      ));

      expect(renderedHrefs(result.container)).toEqual(
        ADMIN_NAVIGATION_LINKS.map((link) => link.href),
      );
    },
  );
});
