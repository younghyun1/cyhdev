# Embedded asset browser caching

Frontend files and their precompressed zstd/gzip representations remain embedded in the Rust binary. Vite emits `.vite/manifest.json`, which is embedded with the files. On the first static request the backend reads that embedded manifest into a fixed inventory; subsequent requests only check membership. No asset directory is opened at request time in production, and no external asset service is required.

Only manifest-listed files under `/assets/` with Vite's default eight-character hash suffix and a supported asset extension receive `Cache-Control: public, max-age=31536000, immutable`. The manifest includes entry scripts, lazy chunks, stylesheets, and imported assets such as fonts and images. Filename appearance alone is insufficient: public files copied into the output are not granted immutable caching unless the build manifest also identifies them. A missing or malformed manifest falls back to revalidation for every file.

HTML, SPA fallback responses, stable-name assets, and the EU5 application's stable filenames retain `public, max-age=0, must-revalidate`. Squaremap tiles and live JSON retain their separate policies. Zstd, gzip, and identity responses keep representation-specific ETags and `Vary: Accept-Encoding`; conditional 304 responses retain the appropriate cache policy. Missing `/assets/` files return 404 with `no-store`, never the SPA HTML shell. Unacceptable encoding responses also use `no-store`.

## Deployment consistency

Deploy HTML and its referenced assets together through the existing frontend-to-backend build. Never replace the contents of an already-published fingerprinted URL. New content gets a new URL, while browsers reuse unchanged content without revalidation. Browsers that cached the previous revalidation headers acquire the new policy on their next response or conditional validation.

The binary contains only its own build's asset set. This change does not retain prior builds. An already-open tab can still request an older lazy chunk after a deployment and receive 404; reloading fetches the current HTML and asset graph. That existing deployment limitation remains. Serving old chunks for a grace period would require a separate retention design; automatic reloads are not introduced because they can discard unsaved input.

## Verification

All 11 static-asset tests and backend library Clippy pass, covering manifest membership, stable-name and HTML revalidation, hash suffixes containing hyphens/underscores, all three encoding variants, distinct ETags, conditional responses, query strings, missing assets, and old builds without a manifest. Frontend type checking and a small unminified Vite development build verifying manifest output and filename shape pass. The full unminified development bundle exceeds the existing 130 KiB gzip budget at 182.7 KiB; the gate was not changed. No release build or deployment was performed.

Vite's [backend integration documentation](https://vite.dev/guide/backend-integration.html) defines the manifest's `file`, `css`, and `assets` fields.
