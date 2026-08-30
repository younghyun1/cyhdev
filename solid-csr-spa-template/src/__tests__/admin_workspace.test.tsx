import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

const { testLocation } = vi.hoisted(() => ({
  testLocation: {
    pathname: "/admin/operations",
    hash: "",
  },
}));

vi.mock("@solidjs/router", () => ({
  useLocation: () => testLocation,
}));

import AdminWorkspace from "../components/admin/AdminWorkspace";
import {
  ADMIN_OPERATION_SECTION_IDS,
  ADMIN_WORKSPACE_LINKS,
} from "../components/admin/navigation";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import { setAuthenticated, setSuperuser } from "../state/auth";
import { setLocaleSignal, setTexts } from "../state/i18n";

const hrefsWithin = (element: Element): ReadonlyArray<string> =>
  Array.from(element.querySelectorAll("a"), (link) => link.getAttribute("href"))
    .filter((href): href is string => href !== null);

describe("administration workspace", () => {
  beforeEach(() => {
    testLocation.pathname = "/admin/operations";
    testLocation.hash = "";
    setAuthenticated(true);
    setSuperuser(true);
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("does not render before both authentication checks authorize access", async () => {
    setAuthenticated(false);
    const result = render(() => (
      <AdminWorkspace>
        <p>Protected content</p>
      </AdminWorkspace>
    ));
    expect(result.container.querySelector(".admin-workspace")).toBeNull();

    setAuthenticated(true);
    setSuperuser(false);
    await Promise.resolve();
    expect(result.container.querySelector(".admin-workspace")).toBeNull();

    setSuperuser(true);
    await Promise.resolve();
    expect(result.container.querySelector(".admin-workspace")).not.toBeNull();
  });

  it("provides equivalent desktop and compact navigation without nested main landmarks", () => {
    const result = render(() => (
      <AdminWorkspace>
        <p>Protected content</p>
      </AdminWorkspace>
    ));
    const sidebar = result.container.querySelector(".admin-workspace-sidebar");
    const compact = result.container.querySelector(
      ".admin-workspace-mobile-navigation",
    );
    if (sidebar === null || compact === null) {
      throw new Error("expected both administration navigation variants");
    }

    const expectedHrefs = ADMIN_WORKSPACE_LINKS.map((link) => link.href);
    expect(hrefsWithin(sidebar)).toEqual(expectedHrefs);
    expect(hrefsWithin(compact)).toEqual(expectedHrefs);
    expect(result.container.querySelector("main")).toBeNull();
  });

  it("contains only admin-specific destinations and bypasses SPA routing for OpenAPI", () => {
    const result = render(() => (
      <AdminWorkspace>
        <p>Protected content</p>
      </AdminWorkspace>
    ));
    const allHrefs = Array.from(
      result.container.querySelectorAll("a"),
      (link) => link.getAttribute("href"),
    );

    expect(allHrefs).not.toContain("/blog/new");
    expect(allHrefs).not.toContain("/photographs");
    expect(allHrefs).not.toContain("/projects");
    expect(allHrefs).not.toContain("/forum");
    const openApiLinks = result.container.querySelectorAll(
      'a[href="/swagger-ui/"]',
    );
    expect(openApiLinks).toHaveLength(2);
    for (const link of openApiLinks) {
      expect(link.getAttribute("rel")).toBe("external");
    }
  });

  it("marks the current route and exact operations fragment", () => {
    testLocation.hash = `#${ADMIN_OPERATION_SECTION_IDS.mediaCleanup}`;
    const result = render(() => (
      <AdminWorkspace>
        <p>Protected content</p>
      </AdminWorkspace>
    ));
    const sidebar = result.container.querySelector(".admin-workspace-sidebar");
    if (sidebar === null) throw new Error("expected administration sidebar");

    expect(
      sidebar
        .querySelector('a[href="/admin/operations"]')
        ?.getAttribute("aria-current"),
    ).toBe("page");
    expect(
      sidebar
        .querySelector(
          `a[href="/admin/operations#${ADMIN_OPERATION_SECTION_IDS.mediaCleanup}"]`,
        )
        ?.getAttribute("aria-current"),
    ).toBe("location");
    expect(
      sidebar
        .querySelector(
          `a[href="/admin/operations#${ADMIN_OPERATION_SECTION_IDS.retention}"]`,
        )
        ?.hasAttribute("aria-current"),
    ).toBe(false);
  });
});
