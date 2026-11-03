/**
 * Typed page content for the About Me and About Blog sections. Each supported
 * locale supplies one module per section that satisfies these types, so a
 * translation cannot omit a page, section, or block that English has. Strings
 * typed as `Inline` accept the markup in `utils/inlineMarkup.ts`; every other
 * string renders literally.
 */
export type Inline = string;

export type StatItem = {
  /** Display value exactly as it should read, for example `97%` or `~20 MB`. */
  readonly value: string;
  /**
   * Optional integer used for the count-up animation. When present, the
   * animated number must equal the numeric part of `value`.
   */
  readonly count?: number;
  readonly prefix?: string;
  readonly suffix?: string;
  readonly label: Inline;
};

export type RoleProject = {
  readonly name: string;
  readonly highlights: ReadonlyArray<Inline>;
};

export type Role = {
  readonly organization: string;
  readonly title: string;
  readonly period: string;
  readonly location: string;
  readonly summary?: Inline;
  readonly highlights: ReadonlyArray<Inline>;
  readonly projects?: ReadonlyArray<RoleProject>;
  readonly tags?: ReadonlyArray<string>;
};

export type LinkCard = {
  readonly title: string;
  readonly href?: string;
  readonly meta?: string;
  readonly text: Inline;
  readonly tags?: ReadonlyArray<string>;
};

export type PipelineStep = {
  readonly label: string;
  readonly detail: Inline;
};

export type BarItem = {
  readonly label: string;
  /** Raw value used for proportional bar length. */
  readonly value: number;
  /** Formatted value shown beside the bar. */
  readonly display: string;
  readonly emphasis?: boolean;
};

export type ContentBlock =
  | { readonly kind: "paragraph"; readonly text: Inline }
  | { readonly kind: "list"; readonly items: ReadonlyArray<Inline> }
  | { readonly kind: "callout"; readonly title?: string; readonly text: Inline }
  | { readonly kind: "stats"; readonly items: ReadonlyArray<StatItem> }
  | { readonly kind: "roles"; readonly roles: ReadonlyArray<Role> }
  | { readonly kind: "cards"; readonly cards: ReadonlyArray<LinkCard> }
  | { readonly kind: "pipeline"; readonly steps: ReadonlyArray<PipelineStep> }
  | {
      readonly kind: "tags";
      readonly groups: ReadonlyArray<{
        readonly label: string;
        readonly items: ReadonlyArray<string>;
      }>;
    }
  | {
      readonly kind: "bars";
      readonly caption: Inline;
      readonly groups: ReadonlyArray<{
        readonly label: string;
        readonly items: ReadonlyArray<BarItem>;
      }>;
    }
  | {
      readonly kind: "table";
      readonly caption?: Inline;
      readonly columns: ReadonlyArray<string>;
      /** Zero-based column indexes rendered right-aligned with tabular numerals. */
      readonly numericColumns?: ReadonlyArray<number>;
      readonly rows: ReadonlyArray<ReadonlyArray<Inline>>;
    };

export type ContentSection = {
  /** Stable fragment identifier shared by every locale. */
  readonly id: string;
  readonly heading: string;
  readonly blocks: ReadonlyArray<ContentBlock>;
};

export type ContentPage = {
  readonly eyebrow: string;
  readonly title: string;
  readonly lede: Inline;
  readonly sections: ReadonlyArray<ContentSection>;
};

/** Shared labels for the page frame, kept with the page copy of each locale. */
export type ContentChrome = {
  /** Tab label for each section's first page; other tabs use page titles. */
  readonly overview: string;
  readonly updated: string;
  readonly onThisPage: string;
  readonly sectionNavigation: string;
  readonly previous: string;
  readonly next: string;
  /** `{minutes}` is replaced with an integer. */
  readonly readingTime: string;
};

export const ABOUT_ME_PAGE_IDS = [
  "overview",
  "experience",
  "projects",
  "background",
] as const;
export type AboutMePageId = (typeof ABOUT_ME_PAGE_IDS)[number];

export const ABOUT_BLOG_PAGE_IDS = [
  "overview",
  "infrastructure",
  "backend",
  "frontend",
  "subsystems",
  "performance",
] as const;
export type AboutBlogPageId = (typeof ABOUT_BLOG_PAGE_IDS)[number];

export type SectionContent<Id extends string> = {
  readonly chrome: ContentChrome;
  /** Month and year the copy was last reviewed, already localized. */
  readonly updated: string;
  readonly pages: Readonly<Record<Id, ContentPage>>;
};

export type AboutMeContent = SectionContent<AboutMePageId>;
export type AboutBlogContent = SectionContent<AboutBlogPageId>;
