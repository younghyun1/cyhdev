import { lazy, Show, type Component } from "solid-js";

import { isAuthenticated, isSuperuser } from "../../state/auth";
import { archiveAccess } from "../../state/kcna_archive_access";

const NotFound = lazy(() => import("../../errors/404"));

/** What a page needs: to read the archive, or to manage it. */
export type ArchiveNeed = "read" | "manage";

/** Null while the session or the server's answer is still unknown. */
const allowed = (need: ArchiveNeed): boolean | null => {
  const authenticated = isAuthenticated();
  if (authenticated === null) return null;
  if (!authenticated) return false;
  if (need === "manage") return isSuperuser();
  return archiveAccess()?.canRead ?? null;
};

/**
 * Guards an archive page. A visitor without access gets the site's Not Found
 * page in place, signed in or not, so the archive is not advertised to
 * someone who cannot open it and nobody is sent to the sign-in page for it.
 */
export function withArchiveAccess(Inner: Component, need: ArchiveNeed = "read"): Component {
  return () => (
    <Show when={allowed(need) !== null} fallback={null}>
      <Show when={allowed(need) === true} fallback={<NotFound />}>
        <Inner />
      </Show>
    </Show>
  );
}
