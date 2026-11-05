import { describe, expect, it } from "vitest";
import { messageDirection } from "./messageDirection";

describe("messageDirection", () => {
  it("lets Arabic win when the sentence starts with a Latin word", () => {
    expect(messageDirection("Rust أم Go؟ النتيجة جيدة جدا")).toBe("rtl");
  });

  it("keeps English with one Arabic word left to right", () => {
    expect(messageDirection("Rust is fast, as سارة said")).toBe("ltr");
  });

  it("ignores URLs when counting", () => {
    expect(messageDirection("https://doc.rust-lang.org/book/ch01-00.html مرجع جيد")).toBe("rtl");
  });

  it("covers Persian and Urdu", () => {
    expect(messageDirection("PostgreSQL ممتاز است؟")).toBe("rtl");
    expect(messageDirection("یہ سرور Rust میں لکھا گیا ہے")).toBe("rtl");
  });

  it("falls back to auto without letters or on a tie", () => {
    expect(messageDirection("12 / 300 :)")).toBe("auto");
    expect(messageDirection("ab اب")).toBe("auto");
  });
});
