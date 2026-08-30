import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import AdminNavigation from "../components/AdminNavigation";
import { ADMIN_DEFAULT_HREF } from "../components/admin/navigation";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import { handleAdminForbiddenResponse } from "../services/api";
import { isSuperuser, setAuthenticated, setSuperuser } from "../state/auth";
import { setLocaleSignal, setTexts } from "../state/i18n";

const renderedHrefs = (container: HTMLElement): ReadonlyArray<string> =>
  Array.from(
    container.querySelectorAll("a"),
    (link) => link.getAttribute("href"),
  ).filter((href): href is string => href !== null);

describe("administration navigation", () => {
  beforeEach(() => {
    setAuthenticated(false);
    setSuperuser(false);
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

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
      <AdminNavigation
        variant="desktop"
        isActive={() => false}
      />
    ));

    expect(renderedHrefs(result.container)).toEqual([]);

    setAuthenticated(false);
    setSuperuser(true);
    expect(renderedHrefs(result.container)).toEqual([]);
  });

  it.each(["desktop", "mobile"] as const)(
    "renders one plain admin link in the %s navigation",
    (variant) => {
      setAuthenticated(true);
      setSuperuser(true);

      const result = render(() => (
        <AdminNavigation
          variant={variant}
          isActive={() => false}
        />
      ));

      expect(renderedHrefs(result.container)).toEqual([ADMIN_DEFAULT_HREF]);
      expect(result.container.querySelector("details")).toBeNull();
      expect(result.container.querySelector("summary")).toBeNull();
    },
  );

  it("marks the single link active across admin routes", () => {
    setAuthenticated(true);
    setSuperuser(true);

    const result = render(() => (
      <AdminNavigation
        variant="desktop"
        isActive={(href) => href === "/admin/authorization"}
      />
    ));

    expect(result.container.querySelector("a")?.getAttribute("aria-current"))
      .toBe("page");
  });
});
