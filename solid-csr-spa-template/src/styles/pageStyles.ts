/**
 * The project's single style source: Tailwind semantic class strings.
 *
 * Colors reference the mode-aware tokens defined in src/index.css
 * (bg-paper, bg-surface, text-ink, border-line, text-accent, ...), so no
 * entry needs dark: variants; the .dark cascade swaps the underlying vars.
 * Radii: rounded-sm for panels/controls, rounded-full only for pills and
 * avatars. Focus visibility comes from the global :focus-visible outline.
 */
export const pageStyles = {
  page: "min-h-screen w-full bg-transparent text-ink transition-colors",
  pageInner: "mx-auto w-full max-w-6xl px-6 py-10",
  pageInnerNarrow: "mx-auto w-full max-w-xl px-6 py-10",
  titleRow: "flex items-center justify-between gap-4",
  title: "text-3xl font-semibold tracking-tight",
  titleSm: "text-2xl font-semibold tracking-tight",
  subtitle: "text-sm text-ink-muted",
  sectionTitle: "text-xl font-semibold tracking-tight",
  sectionSubtitle: "text-sm text-ink-muted",
  card: "rounded-sm border border-line bg-surface",
  cardHeader: "px-6 py-4 border-b border-line",
  cardBody: "px-6 py-6",
  cardFooter: "px-6 py-4 border-t border-line",
  cardPadded: "rounded-sm border border-line bg-surface p-6",
  input:
    "w-full rounded-sm border border-line bg-surface px-3 py-2 text-sm text-ink placeholder:text-ink-faint",
  textarea:
    "w-full rounded-sm border border-line bg-surface px-3 py-2 text-sm text-ink placeholder:text-ink-faint min-h-[120px]",
  select:
    "w-full rounded-sm border border-line bg-surface px-3 py-2 text-sm text-ink",
  buttonPrimary:
    "inline-flex items-center justify-center rounded-sm bg-ink text-paper px-4 py-2 text-sm font-semibold hover:opacity-90 transition disabled:opacity-60 disabled:cursor-not-allowed",
  buttonSecondary:
    "inline-flex items-center justify-center rounded-sm border border-line bg-surface px-4 py-2 text-sm font-semibold text-ink hover:bg-surface-2 transition disabled:opacity-60 disabled:cursor-not-allowed",
  buttonDanger:
    "inline-flex items-center justify-center rounded-sm bg-danger text-paper px-4 py-2 text-sm font-semibold hover:opacity-90 transition disabled:opacity-60 disabled:cursor-not-allowed",
  buttonGhost:
    "inline-flex items-center justify-center rounded-sm px-3 py-2 text-sm font-medium text-ink-muted hover:text-ink hover:bg-surface-2 transition",
  divider: "border-line",
  muted: "text-sm text-ink-muted",
  link: "text-accent hover:text-accent-strong underline underline-offset-4 decoration-accent/40 hover:decoration-accent transition-colors",
  alertError:
    "rounded-sm border border-danger/30 bg-danger/10 px-4 py-3 text-sm text-danger",
  alertSuccess:
    "rounded-sm border border-ok/30 bg-ok/10 px-4 py-3 text-sm text-ok",
  badge:
    "inline-flex items-center rounded-full bg-surface-2 px-2 py-1 text-xs font-semibold text-ink-muted",
  chipBase:
    "inline-flex items-center rounded-full px-2 py-0.5 text-xs font-semibold max-w-full truncate",
  chipQueued: "bg-surface-2 text-ink-muted",
  chipActive: "bg-accent-soft text-accent",
  chipCompleted: "bg-ok/15 text-ok",
  chipFailed: "bg-danger/15 text-danger",
  callPanel: "border-b border-line px-4 py-3 space-y-3",
  callHeader: "flex items-center justify-between gap-2",
  callGrid: "grid gap-2 grid-cols-2 sm:grid-cols-3",
  callTile:
    "relative aspect-video overflow-hidden rounded-sm border border-line bg-black",
  callVideo: "h-full w-full object-cover",
  callTileFallback:
    "absolute inset-0 flex items-center justify-center bg-black/80 text-[#e8e3da]",
  callAvatar:
    "flex h-12 w-12 items-center justify-center rounded-full bg-surface-2 object-cover text-lg font-semibold uppercase text-ink",
  callTileOverlay:
    "absolute inset-x-0 bottom-0 flex items-center justify-between gap-1 bg-black/50 px-2 py-1 text-xs text-white",
  callControls: "flex flex-wrap items-center gap-2",
  callError:
    "rounded-sm border border-danger/30 bg-danger/10 px-3 py-2 text-xs text-danger",
  callPill:
    "inline-flex items-center gap-1 rounded-full bg-ok/15 px-2 py-0.5 text-xs font-semibold text-ok",
  sourceMapTitle: "text-lg font-bold text-ink",
  sourceMapIntro: "mt-2 text-sm leading-6 text-ink-muted",
  sourceGroupGrid: "mt-5 grid gap-4 sm:grid-cols-2",
  sourceGroup: "rounded-sm border border-line bg-surface-2 p-4",
  sourceGroupTitle: "text-sm font-semibold text-ink",
  sourceGroupDescription: "mt-1 text-xs leading-5 text-ink-muted",
  sourceLinkList: "mt-3 space-y-2",
  sourceLink:
    "block rounded-sm border border-line bg-surface px-3 py-2 transition-colors hover:border-line-strong hover:bg-paper",
  sourceLinkLabel: "block text-sm font-medium text-accent",
  sourceLinkPath: "mt-1 block break-all text-xs text-ink-muted",
};

/**
 * Tailwind classes for a batch processing status chip, keyed by the item's
 * status name (`queued` | `encoding` | `uploading` | `persisting` |
 * `completed` | `failed`).
 */
export function chipClass(status: string): string {
  switch (status) {
    case "completed":
      return `${pageStyles.chipBase} ${pageStyles.chipCompleted}`;
    case "failed":
      return `${pageStyles.chipBase} ${pageStyles.chipFailed}`;
    case "queued":
      return `${pageStyles.chipBase} ${pageStyles.chipQueued}`;
    default:
      // encoding / uploading / persisting are all "in progress".
      return `${pageStyles.chipBase} ${pageStyles.chipActive}`;
  }
}
