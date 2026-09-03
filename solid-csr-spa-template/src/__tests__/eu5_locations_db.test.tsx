import { cleanup, render, screen } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

vi.mock("@solidjs/router", () => ({
  useLocation: () => ({ pathname: "/" }),
}));

vi.mock("../services/all_api", () => ({
  authApi: { logout: vi.fn() },
  i18nApi: { getUiTextBundle: vi.fn() },
}));

import PublicNavigation, {
  NAV_ITEMS,
  NAV_LINKS,
} from "../components/PublicNavigation";
import TopBar from "../components/TopBar";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import { KO_KR_DEFAULT_TEXTS } from "../i18n/defaults/ko-kr";
import Eu5LocationsDb from "../pages/eu5_locations_db";
import { setAuthenticated } from "../state/auth";
import { setLocaleSignal, setTexts } from "../state/i18n";

describe("EU5 Locations DB page", () => {
  beforeEach(() => {
    setAuthenticated(false);
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });

  afterEach(() => cleanup());

  it("loads the first-party Slint host eagerly with browser permissions", () => {
    const result = render(() => <Eu5LocationsDb />);
    const frame = result.container.querySelector("iframe");

    expect(frame?.getAttribute("src")).toBe(
      "/eu5-locations-db/app/index.html",
    );
    expect(frame?.getAttribute("title")).toBe("EU5 Locations DB");
    expect(frame?.getAttribute("loading")).toBe("eager");
    expect(frame?.getAttribute("sandbox")).toContain("allow-scripts");
    expect(frame?.getAttribute("sandbox")).toContain("allow-same-origin");
    expect(frame?.getAttribute("sandbox")).toContain("allow-popups");
  });

  it("keeps the exact English label and a Korean translation", () => {
    expect(EN_US_DEFAULT_TEXTS["top_bar.nav.eu5_locations_db"]).toBe(
      "EU5 Locations DB",
    );
    expect(KO_KR_DEFAULT_TEXTS["top_bar.nav.eu5_locations_db"]).toBe(
      "EU5 위치 데이터베이스",
    );
  });

  it("shows the exact public label in desktop and logged-out mobile navigation", () => {
    expect(NAV_LINKS).toContainEqual({
      href: "/eu5-locations-db",
      labelKey: "top_bar.nav.eu5_locations_db",
    });
    expect(
      NAV_ITEMS.filter((item) => item.kind === "group").map(
        (item) => item.labelKey,
      ),
    ).toEqual([
      "top_bar.nav.about_group",
      "top_bar.nav.community_group",
      "top_bar.nav.projects_group",
    ]);

    render(() => <TopBar />);
    const projects = screen.getByText("Projects", { selector: "summary" });
    expect(projects).not.toBeNull();
    expect(
      screen.getByRole("link", { name: "EU5 Locations DB" }),
    ).not.toBeNull();

    render(() => (
      <ul>
        <PublicNavigation variant="mobile" isActive={() => false} />
      </ul>
    ));
    expect(
      screen.getAllByText("Projects", { selector: "summary" }),
    ).toHaveLength(2);
    expect(
      screen.getAllByRole("link", { name: "EU5 Locations DB" }),
    ).toHaveLength(2);
  });
});
