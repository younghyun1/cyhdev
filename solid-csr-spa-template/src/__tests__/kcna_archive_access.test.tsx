import { cleanup, render } from "@solidjs/testing-library";
import { flush } from "solid-js";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { ADMIN_WORKSPACE_LINKS } from "../components/admin/navigation";
import ArchiveEntryCard from "../components/kcna_archive/ArchiveEntryCard";
import { canManageArchive, canViewArchive } from "../components/kcna_archive/access";
import { ARCHIVE_NAV_LINK, NAV_ITEMS, NAV_LINKS, visibleGroupLinks } from "../components/PublicNavigation";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import { setAuthenticated, setSuperuser } from "../state/auth";
import { setLocaleSignal, setTexts } from "../state/i18n";
import { NO_ARCHIVE_ACCESS, setArchiveAccess } from "../state/kcna_archive_access";
import { ARCHIVE_ADMIN_PATH, ARCHIVE_PATH } from "../utils/kcna_archive/routes";

const entryLinks = (container: HTMLElement): HTMLAnchorElement[] =>
  Array.from(container.querySelectorAll<HTMLAnchorElement>(`a[href="${ARCHIVE_PATH}"]`));

const projectsGroup = () => {
  const group = NAV_ITEMS.find(
    (item) => item.kind === "group" && item.labelKey === "top_bar.nav.projects_group",
  );
  if (group?.kind !== "group") throw new Error("the Projects menu is missing");
  return group;
};

beforeEach(() => {
  setAuthenticated(false);
  setSuperuser(false);
  setArchiveAccess(NO_ARCHIVE_ACCESS);
  setLocaleSignal("en-US");
  setTexts(EN_US_DEFAULT_TEXTS);
});

afterEach(() => cleanup());

describe("archive entry on the Projects page", () => {
  it("shows nothing before the session or the server's answer is known", () => {
    setAuthenticated(null);
    setArchiveAccess(null);
    expect(render(() => <ArchiveEntryCard />).container.textContent).toBe("");
    cleanup();
    setAuthenticated(true);
    expect(render(() => <ArchiveEntryCard />).container.textContent).toBe("");
  });

  it("shows nothing to a signed-out visitor, even if a stale answer says yes", () => {
    setArchiveAccess({ canRead: true, adminOnly: false });
    const result = render(() => <ArchiveEntryCard />);
    expect(entryLinks(result.container)).toHaveLength(0);
    expect(result.container.textContent).not.toContain("KCNA");
  });

  it("shows nothing to a member who is not listed", () => {
    setAuthenticated(true);
    expect(render(() => <ArchiveEntryCard />).container.textContent).toBe("");
  });

  it("shows the card without a badge to a listed member", () => {
    setAuthenticated(true);
    setArchiveAccess({ canRead: true, adminOnly: false });
    const result = render(() => <ArchiveEntryCard />);
    const [link] = entryLinks(result.container);
    expect(link?.textContent).toContain("KCNA Archive");
    expect(link?.querySelector(".kcna-badge")).toBeNull();
  });

  it("marks the archive for a superuser while nobody else is listed", () => {
    setAuthenticated(true);
    setSuperuser(true);
    setArchiveAccess({ canRead: true, adminOnly: true });
    const result = render(() => <ArchiveEntryCard />);
    const [link] = entryLinks(result.container);
    expect(link?.querySelector(".kcna-badge")?.textContent).toBe("Administrators only");
  });

  it("appears and disappears as the access changes", async () => {
    setAuthenticated(true);
    setArchiveAccess({ canRead: true, adminOnly: false });
    const result = render(() => <ArchiveEntryCard />);
    expect(entryLinks(result.container)).toHaveLength(1);
    setArchiveAccess(NO_ARCHIVE_ACCESS);
    await Promise.resolve();
    expect(entryLinks(result.container)).toHaveLength(0);
  });
});

describe("archive navigation entries", () => {
  it("is under Projects only for a visitor who may read it", () => {
    expect(NAV_LINKS.some((link) => link.href.startsWith(ARCHIVE_PATH))).toBe(false);
    expect(visibleGroupLinks(projectsGroup()).some((link) => link.href === ARCHIVE_PATH)).toBe(false);
    expect(ARCHIVE_NAV_LINK.visible()).toBe(false);

    setAuthenticated(true);
    setArchiveAccess({ canRead: true, adminOnly: false });
    flush();
    expect(ARCHIVE_NAV_LINK.visible()).toBe(true);
    const links = visibleGroupLinks(projectsGroup());
    expect(links[links.length - 1]).toMatchObject({ href: ARCHIVE_PATH, labelKey: "kcna.title" });
  });

  it("puts the access setting, not the archive, in the admin workspace", () => {
    const hrefs: ReadonlyArray<string> = ADMIN_WORKSPACE_LINKS.map((link) => link.href);
    expect(hrefs).toContain(ARCHIVE_ADMIN_PATH);
    expect(hrefs).not.toContain(ARCHIVE_PATH);
  });

  it("separates reading from managing", () => {
    setAuthenticated(true);
    setArchiveAccess({ canRead: true, adminOnly: false });
    flush();
    expect(canViewArchive()).toBe(true);
    expect(canManageArchive()).toBe(false);
    setSuperuser(true);
    flush();
    expect(canManageArchive()).toBe(true);
  });
});
