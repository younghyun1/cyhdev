import { createEffect } from "solid-js";

import { kcnaArchiveApi } from "../../services/contracts/kcna_archive";
import { isAuthenticated, isSuperuser, user } from "../../state/auth";
import { archiveAccess, NO_ARCHIVE_ACCESS, setArchiveAccess } from "../../state/kcna_archive_access";

/**
 * Whether the archive may be shown to this visitor. Entry points (the Projects
 * menu and card, the command palette) and the route guard use this; the server
 * remains authoritative.
 */
export const canViewArchive = (): boolean =>
  isAuthenticated() === true && archiveAccess()?.canRead === true;

/** Whether this visitor manages the archive: its status, reload, and reader list. */
export const canManageArchive = (): boolean => isAuthenticated() === true && isSuperuser() === true;

/** True while nobody but superusers may read the archive. */
export const archiveIsRestricted = (): boolean => archiveAccess()?.adminOnly === true;

/**
 * Keeps the archive access in step with the session: asks the server once per
 * signed-in account and answers "no" without asking for a signed-out visitor.
 * Called once, from the application root.
 */
export function useArchiveAccess(): void {
  let latest = 0;
  createEffect(
    () => [isAuthenticated(), user()?.user_info?.user_id] as const,
    ([authenticated]) => {
      latest += 1;
      const ticket = latest;
      if (authenticated === null) {
        setArchiveAccess(null);
        return;
      }
      if (!authenticated) {
        setArchiveAccess(NO_ARCHIVE_ACCESS);
        return;
      }
      setArchiveAccess(null);
      // A failure, including an unverified address, reads as no access.
      kcnaArchiveApi.access().then(
        (access) => {
          if (ticket === latest) setArchiveAccess(access);
        },
        () => {
          if (ticket === latest) setArchiveAccess(NO_ARCHIVE_ACCESS);
        },
      );
    },
  );
}
