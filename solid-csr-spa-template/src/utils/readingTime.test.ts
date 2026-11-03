import { afterEach, describe, expect, it } from "vitest";
import { estimateReadingMinutes, outlineHeadings, slugify } from "./readingTime";

afterEach(() => {
  document.body.innerHTML = "";
});

describe("estimateReadingMinutes", () => {
  it("counts words for space-separated languages", () => {
    expect(estimateReadingMinutes("word ".repeat(660), "en-US")).toBe(3);
    expect(estimateReadingMinutes("short", "ko-KR")).toBe(1);
  });

  it("counts characters for Chinese and Japanese", () => {
    expect(estimateReadingMinutes("字".repeat(1500), "zh-Hans")).toBe(3);
    expect(estimateReadingMinutes("字 ".repeat(1000), "ja-JP")).toBe(2);
  });
});

describe("slugify", () => {
  it("keeps letters and digits from any script", () => {
    expect(slugify("Turning a gigabyte of CSV into something")).toBe(
      "turning-a-gigabyte-of-csv-into-something",
    );
    expect(slugify("1) Acquiring a geo-IP database")).toBe("1-acquiring-a-geo-ip-database");
    expect(slugify("데이터 모델")).toBe("데이터-모델");
  });

  it("bounds the fragment length", () => {
    expect(slugify("a".repeat(200))).toHaveLength(64);
  });
});

describe("outlineHeadings", () => {
  it("assigns prefixed, unique ids and keeps existing ones", () => {
    document.body.innerHTML = `
      <article>
        <h2>Setup</h2>
        <h3>Setup</h3>
        <h2 id="kept">Results</h2>
        <h4>Ignored</h4>
        <h2>   </h2>
      </article>`;
    const outline = outlineHeadings(document.body);
    expect(outline).toEqual([
      { id: "post-setup", text: "Setup", level: 2 },
      { id: "post-setup-2", text: "Setup", level: 3 },
      { id: "kept", text: "Results", level: 2 },
    ]);
    expect(document.querySelectorAll("#post-setup")).toHaveLength(1);
  });
});
