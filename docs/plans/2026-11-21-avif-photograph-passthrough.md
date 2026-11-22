# AVIF photographs stored as uploaded

Status: implemented and awaiting review, November 21. Not complete: the Linux gates below have not run. Intended outcome: a photograph exported as AVIF reaches the object store unchanged, with its embedded metadata, while every other format is still encoded to AVIF.

Checkout: branch `feat/avif-upload-passthrough`, based on `origin/main` at `1b950eec`. Implementation commits: `2d8a232a` (backend) and `2137fdad` (frontend). No uncommitted paths.

## Problem

The photograph endpoints refused `image/avif`, and every accepted format was decoded and encoded to AVIF. The encoder writes pixels only, so EXIF, XMP, and colour data were dropped. An AVIF exported from a raw processor had to be converted to another format before upload and then arrived re-encoded, without its capture metadata.

## Decisions

- An AVIF photograph is stored byte-for-byte. The server checks its `ftyp` brand and nothing else. [Media persistence](../architecture/be/media-persistence.md) describes the implemented behaviour.
- No AV1 decoder is added to the server. The two candidates were the `image` crate's `avif-native` feature, which links the C library dav1d and would have to be built statically for the musl target, and the `avif-decode` crate (3.0.0, checked on crates.io on November 21), which pulls `rav1d` with its assembly enabled on x86-64 and so needs `nasm` in all three builder stages. `avif-parse`, which `avif-decode` uses, also rejects grid-based AVIF, and a plain YUV to RGB conversion would give wrong thumbnails for HDR and wide-gamut exports. Either choice would also end the property that no codec outside the upload allowlist is reachable from an upload.
- The thumbnail is instead encoded from an image the browser renders. The browser decodes AVIF natively, handles grids, and converts HDR and wide-gamut sources to sRGB when drawing to a 2D canvas. The rendering is a JPEG of at most 1,600 pixels, twice the 800-pixel thumbnail bound, so the server's resampler does the final reduction.
- The request contract gains one multipart part, `thumbnail`. On the single upload it may appear anywhere; in a batch it directly follows the AVIF `files` part it belongs to. Both request bodies were already free-form multipart in the OpenAPI document, so no annotation or generated client changes.
- AVIF stays refused for profile pictures and demo thumbnails. They must be resized, so a passthrough has no meaning there.
- Other formats are still encoded without their metadata. Carrying EXIF into the encoded file would publish GPS positions that are stripped today; that was not requested and is left alone.
- The map position of a photograph is still the one entered in the upload form. Prefilling it from the file's embedded GPS position is a possible follow-up and is not part of this change.

## Verification

This work was done on a Windows host. The backend does not compile there (27 errors on `origin/main`, all in Unix-only code such as `std::os::unix` and `tokio::net::UnixStream`), and the host has no Node, Docker, or Linux environment, so no `cargo xtask` gate was run.

- `cargo check --locked -p rust-be-template --lib` and the same with `--tests`: no error or warning in any changed file other than the existing Unix-only calls in `util/image/batch_pipeline.rs`, where the new `open_thumbnail_source` uses the same `.mode(0o600)` as `open_item`. Error count went from 27 to 28 for that reason.
- The platform-independent modules were compiled from the worktree into a scratch crate with the same dependency versions and their unit tests run natively: 14 passed. That covers `util/image/avif_original.rs`, `util/image/exif_utils.rs`, `util/image/process_uploaded_image_files.rs`, `util/media/image_upload.rs`, and `features/photography/service/photograph_source.rs`, including storing an AVIF untouched, refusing an AVIF without a thumbnail image, refusing a mislabelled file, and reading the capture time from an AVIF encoded with EXIF.
- `rustfmt --check --edition 2024` on every changed Rust file passed.
- Frontend, through Deno 2.9.8 with dependencies resolved from `package.json` rather than the npm lockfile: `tsc --noEmit` passed under TypeScript 6.0.3, and `vitest run` passed 529 tests in 71 files, including the 8 new ones in `src/utils/avifThumbnail.test.ts`.
- `git diff --check` passed.

Not run: `cargo xtask clippy`, `cargo xtask fmt`, `cargo xtask unit`, `cargo xtask openapi`, `cargo xtask frontend-check` (ESLint did not start under Deno, which resolved a TypeScript version that `typescript-eslint` rejects), the database suites, and every browser test. The unit tests in `features/photography/api/upload_request.rs` and `util/image/batch_pipeline.rs` were type-checked but not executed. No upload was made against a running backend, and no file exported from a raw processor was available to test with; the tests use AVIF files written by the `image` encoder.

## Remaining

Run on a Linux checkout of the branch, in this order: `cargo xtask clippy`, `cargo xtask fmt`, `cargo xtask unit`, `cargo xtask openapi`, `cargo xtask frontend-check`. Then upload one real exported AVIF through the gallery form against a development backend and confirm three things: the stored object is byte-identical to the export, its thumbnail renders, and the capture time is filled in. If the export uses HDR or a wide gamut, also compare the thumbnail's colours with the original.
