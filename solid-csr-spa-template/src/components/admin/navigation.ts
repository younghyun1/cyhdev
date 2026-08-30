import type { UiTextKey } from "../../i18n/keys";

export const ADMIN_DEFAULT_HREF = "/admin/operations";

export const ADMIN_OPERATION_SECTION_IDS = {
  retention: "retention-notifications",
  mediaCleanup: "media-cleanup",
  i18n: "i18n-sync",
  hardPurge: "hard-purge",
} as const;

export type AdminWorkspaceLink = {
  readonly href: string;
  readonly labelKey: UiTextKey;
  readonly depth: 0 | 1;
  readonly external?: true;
};

export const ADMIN_WORKSPACE_LINKS = [
  {
    href: ADMIN_DEFAULT_HREF,
    labelKey: "top_bar.admin.operations",
    depth: 0,
  },
  {
    href: `${ADMIN_DEFAULT_HREF}#${ADMIN_OPERATION_SECTION_IDS.retention}`,
    labelKey: "operations.retention.title",
    depth: 1,
  },
  {
    href: `${ADMIN_DEFAULT_HREF}#${ADMIN_OPERATION_SECTION_IDS.mediaCleanup}`,
    labelKey: "operations.media.title",
    depth: 1,
  },
  {
    href: `${ADMIN_DEFAULT_HREF}#${ADMIN_OPERATION_SECTION_IDS.i18n}`,
    labelKey: "operations.i18n.title",
    depth: 1,
  },
  {
    href: `${ADMIN_DEFAULT_HREF}#${ADMIN_OPERATION_SECTION_IDS.hardPurge}`,
    labelKey: "operations.purge.title",
    depth: 1,
  },
  {
    href: "/admin/authorization",
    labelKey: "top_bar.admin.authorization",
    depth: 0,
  },
  {
    href: "/swagger-ui/",
    labelKey: "top_bar.admin.openapi",
    depth: 0,
    external: true,
  },
] as const satisfies ReadonlyArray<AdminWorkspaceLink>;

export const ADMIN_TOP_BAR_ACTIVE_HREFS = [
  ADMIN_DEFAULT_HREF,
  "/admin/authorization",
] as const;

export function isAdminWorkspaceLinkActive(
  href: string,
  pathname: string,
  hash: string,
): boolean {
  const [targetPath = "", targetFragment] = href.split("#", 2);
  if (targetPath !== pathname) return false;
  if (targetFragment === undefined) return true;
  return normalizeHash(hash) === `#${targetFragment}`;
}

function normalizeHash(hash: string): string {
  if (hash.length === 0 || hash === "#") return "";
  return hash.startsWith("#") ? hash : `#${hash}`;
}
