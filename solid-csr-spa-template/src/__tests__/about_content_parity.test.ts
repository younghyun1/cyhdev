import { describe, expect, it } from "vitest";
import type {
  AboutBlogContent,
  AboutMeContent,
  ContentBlock,
  ContentPage,
  SectionContent,
} from "../pages/about-locales/content_types";
import { parseInline } from "../utils/inlineMarkup";
import { UI_LOCALES } from "../i18n/locales";
import meEn from "../pages/about-locales/about_me_en_us";
import blogEn from "../pages/about-locales/about_blog_en_us";

type Modules<T> = Record<string, { readonly default: T }>;
const meModules: Modules<AboutMeContent> = import.meta.glob("../pages/about-locales/about_me_*.ts", { eager: true });
const blogModules: Modules<AboutBlogContent> = import.meta.glob("../pages/about-locales/about_blog_*.ts", { eager: true });
const entries = <T,>(modules: Modules<T>): ReadonlyArray<readonly [string, T]> =>
  Object.entries(modules).map(([path, module]) => [path, module.default]);

/** Every inline string in a block, in document order. */
function inlineStrings(block: ContentBlock): string[] {
  switch (block.kind) {
    case "paragraph":
      return [block.text];
    case "list":
      return [...block.items];
    case "callout":
      return [block.text];
    case "stats":
      return block.items.map((item) => item.label);
    case "roles":
      return block.roles.flatMap((role) => [
        role.summary ?? "",
        ...role.highlights,
        ...(role.projects ?? []).flatMap((project) => project.highlights),
      ]);
    case "cards":
      return block.cards.map((card) => card.text);
    case "pipeline":
      return block.steps.map((step) => step.detail);
    case "tags":
      return [];
    case "bars":
      return [block.caption];
    case "table":
      return [block.caption ?? "", ...block.rows.flat()];
  }
}

/** Language-independent shape: ids, kinds, link targets, and numbers. */
function skeleton(page: ContentPage): unknown {
  return page.sections.map((section) => ({
    id: section.id,
    blocks: section.blocks.map((block) => ({
      kind: block.kind,
      links: inlineStrings(block).flatMap((text) =>
        parseInline(text).flatMap((segment) => (segment.kind === "link" ? [segment.href] : [])),
      ),
      cards: block.kind === "cards" ? block.cards.map((card) => card.href ?? null) : undefined,
      counts: block.kind === "stats" ? block.items.map((item) => item.count ?? null) : undefined,
      bars:
        block.kind === "bars"
          ? block.groups.map((group) => group.items.map((item) => [item.value, item.emphasis ?? false]))
          : undefined,
      tags: block.kind === "tags" ? block.groups.map((group) => group.items) : undefined,
      size:
        block.kind === "roles"
          ? block.roles.map((role) => [role.highlights.length, role.projects?.length ?? 0])
          : block.kind === "table"
            ? [block.columns.length, block.rows.length]
            : undefined,
    })),
  }));
}

function checkSection<Id extends string>(
  english: SectionContent<Id>,
  translations: ReadonlyArray<readonly [string, SectionContent<Id>]>,
) {
  for (const [tag, translated] of translations) {
    for (const id of Object.keys(english.pages) as Id[]) {
      expect(skeleton(translated.pages[id]), `${tag} ${id}`).toEqual(skeleton(english.pages[id]));
      for (const section of translated.pages[id].sections) {
        for (const block of section.blocks) {
          if (block.kind !== "stats") continue;
          for (const item of block.items) {
            if (item.count === undefined) continue;
            expect(`${item.prefix ?? ""}${item.count}${item.suffix ?? ""}`, `${tag} ${id}`).toBe(item.value);
          }
        }
      }
    }
    expect(translated.chrome.readingTime, tag).toContain("{minutes}");
  }
}

describe("About content translations", () => {
  it("exist for every interface language", () => {
    expect(Object.keys(meModules)).toHaveLength(UI_LOCALES.length);
    expect(Object.keys(blogModules)).toHaveLength(UI_LOCALES.length);
  });

  it("match the English About Me structure", () => {
    checkSection(meEn, entries(meModules));
  });

  it("match the English About Blog structure", () => {
    checkSection(blogEn, entries(blogModules));
  });

  it("keep excluded personal topics out of every locale", () => {
    const all = JSON.stringify(entries(meModules));
    for (const forbidden of [/West Papua/i, /Sakartvelo/i, /human rights/i, /719/, /0000 0000/, /22KR/]) {
      expect(all).not.toMatch(forbidden);
    }
  });
});
