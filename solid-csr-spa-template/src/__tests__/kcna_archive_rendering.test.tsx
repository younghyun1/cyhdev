import { cleanup, render } from "@solidjs/testing-library";
import { afterEach, beforeEach, describe, expect, it } from "vitest";

import Reader from "../components/kcna_archive/Reader";
import Segments from "../components/kcna_archive/Segments";
import { Notice } from "../components/kcna_archive/Notice";
import { EN_US_DEFAULT_TEXTS } from "../i18n/defaults/en-us";
import type { ArchiveDocumentResponse } from "../services/contracts/kcna_archive";
import { setLocaleSignal, setTexts } from "../state/i18n";

const document_: ArchiveDocumentResponse = {
  id: "0198f4d0-aaaa-7000-8000-000000000001",
  title: [
    { text: "<script>bad()</script> ", hit: false },
    { text: "다리", hit: true },
  ],
  paragraphs: [[{ text: '<img src=x onerror="bad()">', hit: false }]],
  publicationDate: "2026-09-02",
  publicationPrecision: "day",
  language: "kor",
  source: "kcna_kp",
  sources: ["kcna_kp"],
  canonicalUrl: "http://www.kcna.kp/kp/article/q/1.kcmsf",
  category: null,
  sourceRecordId: null,
  metadata: null,
  translations: [{ id: "0198f4d0-aaaa-7000-8000-000000000002", language: "eng", title: "Bridge" }],
  copies: [{ id: "0198f4d0-aaaa-7000-8000-000000000003", source: "kcna_co_jp", canonicalUrl: "http://www.kcna.co.jp/a.html" }],
};

describe("archive text rendering", () => {
  beforeEach(() => {
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });
  afterEach(() => cleanup());

  it("renders matches as marks and everything else as plain text", () => {
    const result = render(() => <Segments segments={document_.title} />);
    expect(result.container.querySelectorAll("mark")).toHaveLength(1);
    expect(result.container.querySelector("mark")?.textContent).toBe("다리");
    expect(result.container.querySelector("script")).toBeNull();
    expect(result.container.textContent).toContain("<script>bad()</script>");
  });

  it("gives the article its own language, reads left to right, and offers no stored-page link", () => {
    const result = render(() => (
      <Reader
        document={document_}
        beside={undefined}
        besideLanguage={undefined}
        hrefFor={(id) => `/x/${id}`}
        onBeside={() => {}}
      />
    ));
    const title = result.container.querySelector(".kcna-document-title");
    expect(title?.getAttribute("lang")).toBe("kor");
    expect(title?.getAttribute("dir")).toBe("ltr");
    const body = result.container.querySelector(".kcna-document-body");
    expect(body?.getAttribute("lang")).toBe("kor");
    expect(body?.getAttribute("dir")).toBe("ltr");
    expect(result.container.querySelector("img")).toBeNull();
    const hrefs = Array.from(result.container.querySelectorAll("a"), (link) => link.getAttribute("href"));
    expect(hrefs.some((href) => href?.includes("/raw"))).toBe(false);
    const source = Array.from(result.container.querySelectorAll("a")).find((link) => link.textContent === "Source page");
    expect(source?.getAttribute("href")).toBe(document_.canonicalUrl);
    expect(source?.getAttribute("rel")).toBe("noreferrer noopener");
    expect(source?.getAttribute("target")).toBe("_blank");
  });

  it("shows both editions in rows when a translation is open, each in its language", () => {
    const beside: ArchiveDocumentResponse = {
      ...document_,
      id: "0198f4d0-aaaa-7000-8000-000000000002",
      language: "eng",
      title: [{ text: "Bridge", hit: false }],
      paragraphs: [[{ text: "English text", hit: false }]],
    };
    const result = render(() => (
      <Reader
        document={document_}
        beside={beside}
        besideLanguage="eng"
        hrefFor={(id) => `/x/${id}`}
        onBeside={() => {}}
      />
    ));
    const rows = result.container.querySelectorAll(".kcna-parallel-row");
    expect(rows).toHaveLength(1);
    expect(rows[0]?.children[0]?.getAttribute("lang")).toBe("kor");
    expect(rows[0]?.children[1]?.getAttribute("lang")).toBe("eng");
    expect(result.container.querySelector("[aria-pressed='true']")?.textContent).toBe("Close side by side");
  });
});

describe("archive notices", () => {
  beforeEach(() => {
    setLocaleSignal("en-US");
    setTexts(EN_US_DEFAULT_TEXTS);
  });
  afterEach(() => cleanup());

  it("reads a missing snapshot as a status and a real failure as an alert", () => {
    const state = render(() => <Notice kind="notInstalled" />);
    expect(state.container.querySelector("[role='status']")?.textContent).toContain("not installed");
    expect(state.container.querySelector("[role='alert']")).toBeNull();
    cleanup();
    const failure = render(() => <Notice kind="network" onRetry={() => {}} />);
    expect(failure.container.querySelector("[role='alert']")?.textContent).toContain("could not be reached");
    expect(failure.container.querySelector("button")?.textContent).toBe("Retry");
  });
});
