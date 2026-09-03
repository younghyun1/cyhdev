import { gzipSync } from "node:zlib";
import type { OutputAsset, OutputBundle, OutputChunk } from "rollup";
import type { Plugin } from "vite";

export const INITIAL_ASSET_BUDGET_BYTES = 130 * 1024;

type ViteOutputChunk = OutputChunk & {
  readonly viteMetadata?: {
    readonly importedCss?: ReadonlySet<string>;
  };
};

function emittedBytes(item: OutputAsset | OutputChunk): string | Uint8Array {
  if (item.type === "chunk") return item.code;
  return typeof item.source === "string" ? item.source : item.source;
}

export function collectInitialAssetNames(bundle: OutputBundle): ReadonlySet<string> {
  const initialAssets = new Set<string>();
  const visit = (fileName: string) => {
    if (initialAssets.has(fileName)) return;
    const item = bundle[fileName];
    if (!item || item.type !== "chunk") return;
    initialAssets.add(fileName);
    for (const imported of item.imports) visit(imported);
    const metadata = (item as ViteOutputChunk).viteMetadata;
    for (const stylesheet of metadata?.importedCss ?? []) {
      if (bundle[stylesheet]?.type === "asset") initialAssets.add(stylesheet);
    }
  };

  for (const item of Object.values(bundle)) {
    if (item.type === "chunk" && item.isEntry) visit(item.fileName);
  }
  return initialAssets;
}

export function initialAssetBudgetPlugin(
  limitBytes = INITIAL_ASSET_BUDGET_BYTES,
): Plugin {
  return {
    name: "initial-asset-budget",
    apply: "build",
    enforce: "post",
    generateBundle(_options, bundle) {
      const names = collectInitialAssetNames(bundle);
      const gzipBytes = [...names].reduce((sum, fileName) => {
        const item = bundle[fileName];
        return item ? sum + gzipSync(emittedBytes(item), { level: 9 }).byteLength : sum;
      }, 0);
      const kib = (gzipBytes / 1024).toFixed(1);
      const limitKib = (limitBytes / 1024).toFixed(0);
      this.info(`initial module graph: ${kib} KiB gzip (${names.size} assets)`);
      if (gzipBytes > limitBytes) {
        this.error(
          `Initial module graph is ${kib} KiB gzip; budget is ${limitKib} KiB.`,
        );
      }
    },
  };
}
