import { describe, expect, it } from "vitest";
import { formatBuildTime, postgresVersion, visitorPopupText } from "./statusMetadata";

describe("status presentation", () => {
  it("normalizes frontend and backend timestamps to UTC", () => {
    expect(formatBuildTime("2026-09-22T05:20:27.000Z")).toBe("2026-09-22 05:20:27 UTC");
    expect(formatBuildTime("2026-09-22T05:20:27+00:00")).toBe("2026-09-22 05:20:27 UTC");
    expect(formatBuildTime(undefined)).toBe("…");
    expect(formatBuildTime("invalid")).toBe("…");
  });
  it("names the database without repeating its product name", () => {
    expect(postgresVersion("18.4")).toBe("PostgreSQL 18.4");
    expect(postgresVersion("PostgreSQL 18.4")).toBe("PostgreSQL 18.4");
  });
  it("removes legacy bold markup without interpreting other markup", () => {
    expect(visitorPopupText("Visitations from here: <b>6</b>")).toBe("Visitations from here: 6");
    expect(visitorPopupText('<img src=x onerror="alert(1)"><b>6</b>')).toBe('<img src=x onerror="alert(1)">6');
  });
});
