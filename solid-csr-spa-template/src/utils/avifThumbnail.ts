// AVIF photographs are stored by the server exactly as uploaded, which keeps
// their embedded metadata. The server has no AV1 decoder, so it cannot make
// the gallery thumbnail from such a file. The browser can decode it: each AVIF
// is rendered here to a small JPEG, which travels in the same request and is
// the only source of that photograph's thumbnail.

const AVIF_MIME = "image/avif";
const AVIF_EXTENSION = /\.avif$/i;

// Twice the server's 800-pixel thumbnail bound, so the server's resampler
// still has detail to work with after the browser's own downscale.
export const THUMBNAIL_SOURCE_MAX_EDGE = 1600;
const THUMBNAIL_SOURCE_QUALITY = 0.92;

// Multipart field the server pairs with the AVIF part sent just before it.
const THUMBNAIL_FIELD = "thumbnail";

export interface PixelSize {
  readonly width: number;
  readonly height: number;
}

export type ThumbnailRenderer = (image: Blob) => Promise<Blob>;

// The declared type decides when the platform supplies one. Some systems have
// no MIME mapping for `.avif` and report an empty type, so the name decides
// there.
export const isAvifImage = (image: Blob, fileName: string): boolean =>
  image.type === ""
    ? AVIF_EXTENSION.test(fileName)
    : image.type.toLowerCase() === AVIF_MIME;

// Dimensions that fit within the thumbnail source bound, keeping the aspect
// ratio. An image already inside the bound is never enlarged.
export const thumbnailSourceSize = (width: number, height: number): PixelSize => {
  const longEdge = Math.max(width, height);
  if (longEdge <= THUMBNAIL_SOURCE_MAX_EDGE) return { width, height };
  const scale = THUMBNAIL_SOURCE_MAX_EDGE / longEdge;
  return {
    width: Math.max(1, Math.round(width * scale)),
    height: Math.max(1, Math.round(height * scale)),
  };
};

// Decodes an image the browser supports and renders it as a JPEG bounded by
// THUMBNAIL_SOURCE_MAX_EDGE. The canvas is 8-bit sRGB, so wide-gamut and HDR
// sources are converted by the browser's colour management on the way in.
export const renderThumbnailSource: ThumbnailRenderer = async (image) => {
  const bitmap = await createImageBitmap(image);
  try {
    const size = thumbnailSourceSize(bitmap.width, bitmap.height);
    const canvas = document.createElement("canvas");
    canvas.width = size.width;
    canvas.height = size.height;
    const context = canvas.getContext("2d");
    if (!context) throw new Error("2D canvas is unavailable");
    context.imageSmoothingEnabled = true;
    context.imageSmoothingQuality = "high";
    // JPEG has no alpha channel; without a backdrop, transparent pixels
    // would come out black.
    context.fillStyle = "#ffffff";
    context.fillRect(0, 0, size.width, size.height);
    context.drawImage(bitmap, 0, 0, size.width, size.height);
    return await new Promise<Blob>((resolve, reject) => {
      canvas.toBlob(
        (blob) => {
          if (blob) resolve(blob);
          else reject(new Error("Canvas could not encode the thumbnail image"));
        },
        "image/jpeg",
        THUMBNAIL_SOURCE_QUALITY,
      );
    });
  } finally {
    // A decoded photograph holds four bytes per pixel until it is closed.
    bitmap.close();
  }
};

// Appends one photograph to an upload form under `fieldName`. An AVIF is
// followed directly by its rendered thumbnail image: the server pairs the two
// by position. The thumbnail is rendered before anything is appended, so a
// failed render leaves the form without a half-added photograph.
export const appendPhotograph = async (
  formData: FormData,
  fieldName: string,
  image: Blob,
  fileName: string,
  render: ThumbnailRenderer = renderThumbnailSource,
): Promise<void> => {
  if (!isAvifImage(image, fileName)) {
    formData.append(fieldName, image, fileName);
    return;
  }
  const thumbnail = await render(image);
  // The server accepts the part only under the AVIF content type, which the
  // platform may have left empty.
  const original =
    image.type.toLowerCase() === AVIF_MIME
      ? image
      : new Blob([image], { type: AVIF_MIME });
  formData.append(fieldName, original, fileName);
  formData.append(THUMBNAIL_FIELD, thumbnail, `${fileName}.thumbnail.jpg`);
};
