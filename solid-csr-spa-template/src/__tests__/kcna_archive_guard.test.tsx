import { cleanup, render } from "@solidjs/testing-library";
import { createRoot, flush, Loading } from "solid-js";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";

import { useArchiveAccess } from "../components/kcna_archive/access";
import { withArchiveAccess } from "../components/kcna_archive/RequireArchiveAccess";
import { setAuthenticated, setSuperuser, setUser } from "../state/auth";
import { archiveAccess, NO_ARCHIVE_ACCESS, setArchiveAccess } from "../state/kcna_archive_access";

// The real page reads the router; the guard only needs to mount something in its place.
vi.mock("../errors/404", () => ({ default: () => <p data-testid="not-found">Not found</p> }));

const Page = () => <p data-testid="archive-page">Archive</p>;
const Reading = withArchiveAccess(Page);
const Managing = withArchiveAccess(Page, "manage");

const shown = async (Guarded: typeof Reading): Promise<string[]> => {
  const result = render(() => (
    <Loading fallback={null}>
      <Guarded />
    </Loading>
  ));
  // The Not Found page is loaded lazily.
  await vi.dynamicImportSettled();
  flush();
  return Array.from(result.container.querySelectorAll("[data-testid]")).map(
    (element) => element.getAttribute("data-testid") ?? "",
  );
};

beforeEach(() => {
  setAuthenticated(false);
  setSuperuser(false);
  setUser(null);
  setArchiveAccess(NO_ARCHIVE_ACCESS);
});

afterEach(() => {
  cleanup();
  vi.unstubAllGlobals();
});

describe("archive route guard", () => {
  it("shows nothing while the session or the server's answer is unknown", async () => {
    setAuthenticated(null);
    expect(await shown(Reading)).toEqual([]);
    cleanup();
    setAuthenticated(true);
    setArchiveAccess(null);
    expect(await shown(Reading)).toEqual([]);
  });

  it("answers a signed-out visitor with Not Found, never a redirect", async () => {
    expect(await shown(Reading)).toEqual(["not-found"]);
    cleanup();
    expect(await shown(Managing)).toEqual(["not-found"]);
  });

  it("answers a member who is not listed with Not Found", async () => {
    setAuthenticated(true);
    expect(await shown(Reading)).toEqual(["not-found"]);
  });

  it("lets a listed member read but not manage", async () => {
    setAuthenticated(true);
    setArchiveAccess({ canRead: true, adminOnly: false });
    expect(await shown(Reading)).toEqual(["archive-page"]);
    cleanup();
    expect(await shown(Managing)).toEqual(["not-found"]);
  });

  it("lets a superuser read and manage", async () => {
    setAuthenticated(true);
    setSuperuser(true);
    setArchiveAccess({ canRead: true, adminOnly: true });
    expect(await shown(Reading)).toEqual(["archive-page"]);
    cleanup();
    expect(await shown(Managing)).toEqual(["archive-page"]);
  });
});

describe("archive access follows the session", () => {
  const answer = (data: unknown, status = 200) => {
    const fetchMock = vi.fn(
      async (_input: RequestInfo | URL, _init?: RequestInit) =>
        new Response(JSON.stringify({ success: status === 200, data, meta: {} }), { status }),
    );
    vi.stubGlobal("fetch", fetchMock);
    return fetchMock;
  };

  const settle = async (): Promise<void> => {
    flush();
    await new Promise((resolve) => setTimeout(resolve, 0));
    flush();
  };

  it("does not ask the server for a signed-out visitor", async () => {
    const fetchMock = answer({ canRead: true, adminOnly: false });
    setArchiveAccess(null);
    const dispose = createRoot((dispose) => {
      useArchiveAccess();
      return dispose;
    });
    await settle();
    expect(fetchMock).not.toHaveBeenCalled();
    expect(archiveAccess()).toEqual(NO_ARCHIVE_ACCESS);
    dispose();
  });

  it("asks once per signed-in account and drops the answer on sign-out", async () => {
    const fetchMock = answer({ canRead: true, adminOnly: false });
    const dispose = createRoot((dispose) => {
      useArchiveAccess();
      return dispose;
    });
    setAuthenticated(true);
    await settle();
    expect(fetchMock).toHaveBeenCalledTimes(1);
    expect(archiveAccess()).toEqual({ canRead: true, adminOnly: false });

    setAuthenticated(false);
    await settle();
    expect(archiveAccess()).toEqual(NO_ARCHIVE_ACCESS);
    expect(fetchMock).toHaveBeenCalledTimes(1);
    dispose();
  });

  it("reads a refusal, such as an unverified address, as no access", async () => {
    answer({}, 400);
    const dispose = createRoot((dispose) => {
      useArchiveAccess();
      return dispose;
    });
    setAuthenticated(true);
    await settle();
    expect(archiveAccess()).toEqual(NO_ARCHIVE_ACCESS);
    dispose();
  });
});
