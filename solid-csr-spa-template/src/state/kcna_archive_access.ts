import { createSignal } from "solid-js";

/** What the server says about this visitor and the archive. */
export interface ArchiveAccess {
  readonly canRead: boolean;
  /** True for a superuser while nobody else is allowed to read. */
  readonly adminOnly: boolean;
}

/** The answer for a visitor who is signed out or whom the server turned away. */
export const NO_ARCHIVE_ACCESS: ArchiveAccess = { canRead: false, adminOnly: false };

/** Null until the session has resolved and the server has answered. */
export const [archiveAccess, setArchiveAccess] = createSignal<ArchiveAccess | null>(null);
