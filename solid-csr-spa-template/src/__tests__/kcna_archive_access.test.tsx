import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import { ADMIN_WORKSPACE_LINKS } from "../components/admin/navigation";
import ArchiveEntryCard from "../components/kcna_archive/ArchiveEntryCard";
import { NAV_LINKS } from "../components/PublicNavigation";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import { setAuthenticated, setSuperuser } from "../state/auth";
import { setLocaleSignal, setTexts } from "../state/i18n";
import { ARCHIVE_PATH } from "../utils/kcna_archive/routes";

const entryLinks = (container: HTMLElement): HTMLAnchorElement[] =>
  Array.from(container.querySelectorAll<HTMLAnchorElement>(`a[href="${ARCHIVE_PATH}"]`));

describe("archive entry on the Projects page", () => {
  beforeEach(() => {
    setAuthenticated(false);
    setSuperuser(false);
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("shows nothing before the session resolves", () => {
    setAuthenticated(null);
    setSuperuser(null);
    const result = render(() => <ArchiveEntryCard />);
    expect(result.container.textContent).toBe("");
  });

  it("shows nothing to a signed-out visitor, even if a stale role flag is set", () => {
    setAuthenticated(false);
    setSuperuser(true);
    const result = render(() => <ArchiveEntryCard />);
    expect(entryLinks(result.container)).toHaveLength(0);
    expect(result.container.textContent).not.toContain("KCNA");
  });

  it("shows nothing to a signed-in user who is not a superuser", () => {
    setAuthenticated(true);
    setSuperuser(false);
    const result = render(() => <ArchiveEntryCard />);
    expect(result.container.textContent).toBe("");
  });

  it("shows the card with the administrators-only badge to a superuser", () => {
    setAuthenticated(true);
    setSuperuser(true);
    const result = render(() => <ArchiveEntryCard />);
    const [link] = entryLinks(result.container);
    expect(link?.textContent).toContain("KCNA Archive");
    expect(link?.querySelector(".kcna-badge")?.textContent).toBe("Administrators only");
  });

  it("appears and disappears as the role changes", async () => {
    setAuthenticated(true);
    setSuperuser(true);
    const result = render(() => <ArchiveEntryCard />);
    expect(entryLinks(result.container)).toHaveLength(1);
    setSuperuser(false);
    await Promise.resolve();
    expect(entryLinks(result.container)).toHaveLength(0);
  });
});

describe("archive navigation entries", () => {
  it("is in the admin workspace links and absent from the public navigation", () => {
    expect(ADMIN_WORKSPACE_LINKS.some((link) => link.href === ARCHIVE_PATH)).toBe(true);
    expect(NAV_LINKS.some((link) => link.href.startsWith(ARCHIVE_PATH))).toBe(false);
  });
});
