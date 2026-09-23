import { describe, expect, it } from "vitest";
import { createImagePreloader } from "../components/photographs/imagePreloader";

/** Detached image double whose decode settles when the test says so. */
class FakeImage {
  static created: FakeImage[] = [];
  decoding = "auto";
  fetchPriority = "auto";
  src = "";
  aborted = false;
  private settle: { resolve: () => void; reject: (error: Error) => void } | null = null;

  constructor() {
    FakeImage.created.push(this);
  }

  decode(): Promise<void> {
    return new Promise<void>((resolve, reject) => {
      this.settle = { resolve, reject };
    });
  }

  removeAttribute(name: string): void {
    if (name !== "src") return;
    this.aborted = true;
    this.src = "";
    this.settle?.reject(new Error("aborted"));
  }

  finish(): void {
    this.settle?.resolve();
  }

  fail(): void {
    this.settle?.reject(new Error("failed"));
  }
}

function preloader(maxEntries = 3) {
  FakeImage.created = [];
  return createImagePreloader({
    maxEntries,
    createImage: () => new FakeImage() as unknown as HTMLImageElement,
  });
}

const settled = (promise: Promise<void>) =>
  promise.then(
    () => "ready",
    () => "failed",
  );

describe("image preloader", () => {
  it("shares one request per URL and sets decode hints", async () => {
    const images = preloader();
    const first = images.load("/a.avif", "high");
    const second = images.load("/a.avif", "high");
    expect(second).toBe(first);
    expect(FakeImage.created).toHaveLength(1);
    const image = FakeImage.created[0]!;
    expect(image.src).toBe("/a.avif");
    expect(image.decoding).toBe("async");
    expect(image.fetchPriority).toBe("high");
    image.finish();
    await expect(settled(first)).resolves.toBe("ready");
  });

  it("warms neighbours at low priority without duplicating tracked URLs", () => {
    const images = preloader();
    void images.load("/a.avif");
    images.warm("/a.avif");
    images.warm("/b.avif");
    expect(FakeImage.created.map((image) => image.src)).toEqual(["/a.avif", "/b.avif"]);
    expect(FakeImage.created[1]!.fetchPriority).toBe("low");
  });

  it("evicts and aborts the least recently requested entry beyond the bound", () => {
    const images = preloader(2);
    void settled(images.load("/a.avif"));
    void settled(images.load("/b.avif"));
    // Touch /a so /b becomes the oldest.
    void settled(images.load("/a.avif"));
    void settled(images.load("/c.avif"));
    expect(images.size).toBe(2);
    const [a, b, c] = FakeImage.created;
    expect(b!.aborted).toBe(true);
    expect(a!.aborted).toBe(false);
    expect(c!.aborted).toBe(false);
  });

  it("aborts everything outside the retained set and on clear", () => {
    const images = preloader();
    void settled(images.load("/a.avif"));
    images.warm("/b.avif");
    images.warm("/c.avif");
    images.retain(["/b.avif"]);
    expect(FakeImage.created.map((image) => image.aborted)).toEqual([true, false, true]);
    expect(images.size).toBe(1);
    images.clear();
    expect(FakeImage.created[1]!.aborted).toBe(true);
    expect(images.size).toBe(0);
  });

  it("retries a failed URL on the next load but not on warm-up", async () => {
    const images = preloader();
    const first = images.load("/broken.avif");
    FakeImage.created[0]!.fail();
    await expect(settled(first)).resolves.toBe("failed");

    images.warm("/broken.avif");
    expect(FakeImage.created).toHaveLength(1);

    const retry = images.load("/broken.avif");
    expect(FakeImage.created).toHaveLength(2);
    FakeImage.created[1]!.finish();
    await expect(settled(retry)).resolves.toBe("ready");
  });
});
