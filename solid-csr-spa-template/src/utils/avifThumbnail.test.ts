import { describe, expect, it } from "vitest";
import {
  THUMBNAIL_SOURCE_MAX_EDGE,
  appendPhotograph,
  isAvifImage,
  thumbnailSourceSize,
} from "./avifThumbnail";

const rendering = new Blob(["rendered"], { type: "image/jpeg" });
const renderStub = () => Promise.resolve(rendering);

const partNames = (formData: FormData): string[] =>
  Array.from(formData.entries(), ([name]) => name);

const filePart = (formData: FormData, name: string): File => {
  const value = formData.get(name);
  if (!(value instanceof File)) throw new Error(`${name} is not a file part`);
  return value;
};

describe("isAvifImage", () => {
  it("goes by the declared type when there is one", () => {
    expect(isAvifImage(new Blob([], { type: "image/avif" }), "photo.bin")).toBe(true);
    expect(isAvifImage(new Blob([], { type: "IMAGE/AVIF" }), "photo")).toBe(true);
    expect(isAvifImage(new Blob([], { type: "image/jpeg" }), "photo.avif")).toBe(false);
  });

  it("falls back to the extension when the platform reports no type", () => {
    expect(isAvifImage(new Blob([]), "DSC_0001.AVIF")).toBe(true);
    expect(isAvifImage(new Blob([]), "DSC_0001.jpg")).toBe(false);
    expect(isAvifImage(new Blob([]), "avif")).toBe(false);
  });
});

describe("thumbnailSourceSize", () => {
  it("scales the long edge to the bound and keeps the aspect ratio", () => {
    expect(thumbnailSourceSize(8000, 6000)).toEqual({ width: 1600, height: 1200 });
    expect(thumbnailSourceSize(4000, 6000)).toEqual({ width: 1067, height: 1600 });
  });

  it("never enlarges and never collapses an edge to zero", () => {
    expect(thumbnailSourceSize(640, 480)).toEqual({ width: 640, height: 480 });
    expect(
      thumbnailSourceSize(THUMBNAIL_SOURCE_MAX_EDGE, THUMBNAIL_SOURCE_MAX_EDGE),
    ).toEqual({ width: 1600, height: 1600 });
    expect(thumbnailSourceSize(16000, 2)).toEqual({ width: 1600, height: 1 });
  });
});

describe("appendPhotograph", () => {
  it("sends a decodable format alone, without rendering", async () => {
    const formData = new FormData();
    let rendered = 0;
    await appendPhotograph(
      formData,
      "files",
      new Blob(["jpeg"], { type: "image/jpeg" }),
      "a.jpg",
      () => {
        rendered += 1;
        return Promise.resolve(rendering);
      },
    );
    expect(partNames(formData)).toEqual(["files"]);
    expect(filePart(formData, "files").name).toBe("a.jpg");
    expect(rendered).toBe(0);
  });

  it("follows each AVIF directly with its rendered thumbnail", async () => {
    const formData = new FormData();
    const avif = new Blob(["avif"], { type: "image/avif" });
    const jpeg = new Blob(["jpeg"], { type: "image/jpeg" });
    await appendPhotograph(formData, "files", avif, "a.avif", renderStub);
    await appendPhotograph(formData, "files", jpeg, "b.jpg", renderStub);
    await appendPhotograph(formData, "files", avif, "c.avif", renderStub);
    expect(partNames(formData)).toEqual([
      "files",
      "thumbnail",
      "files",
      "files",
      "thumbnail",
    ]);
    const thumbnails = formData.getAll("thumbnail");
    expect(thumbnails).toHaveLength(2);
    const first = filePart(formData, "thumbnail");
    expect(first.name).toBe("a.avif.thumbnail.jpg");
    expect(first.type).toBe("image/jpeg");
  });

  it("uploads the original AVIF bytes and declares their type", async () => {
    const formData = new FormData();
    await appendPhotograph(formData, "file", new Blob(["avif-bytes"]), "a.avif", renderStub);
    const original = filePart(formData, "file");
    expect(original.type).toBe("image/avif");
    expect(original.name).toBe("a.avif");
    expect(await original.text()).toBe("avif-bytes");
  });

  it("adds nothing when the thumbnail cannot be rendered", async () => {
    const formData = new FormData();
    await expect(
      appendPhotograph(
        formData,
        "files",
        new Blob(["avif"], { type: "image/avif" }),
        "a.avif",
        () => Promise.reject(new Error("decode failed")),
      ),
    ).rejects.toThrow("decode failed");
    expect(partNames(formData)).toEqual([]);
  });
});
