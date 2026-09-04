/// <reference types="vitest" />
import { defineConfig } from "vite";
import solidPlugin from "@solidjs/vite-plugin";
import pkg from "./package.json" with { type: "json" };
import { resolveConfiguredBuildTimestamp } from "./src/config/buildTimestamp.ts";
import { initialAssetBudgetPlugin } from "./build/initialAssetBudget.ts";

export default defineConfig({
  plugins: [solidPlugin(), initialAssetBudgetPlugin()],
  define: {
    __BUILD_TIMESTAMP__: JSON.stringify(
      resolveConfiguredBuildTimestamp(
        process.env.APP_BUILD_EPOCH,
        process.env.SOURCE_DATE_EPOCH,
      ),
    ),
    __SOLID_VERSION__: JSON.stringify(pkg.dependencies["solid-js"] || ""),
    __APP_NAME__: JSON.stringify(pkg.name),
  },
  server: {
    port: 3000,
  },
  build: {
    target: "esnext",
    // The editor is already isolated behind a lazy route; its self-contained
    // Toast UI chunk is ~551 kB minified and does not affect initial loading.
    chunkSizeWarningLimit: 600,
  },
  test: {
    environment: "jsdom",
    globals: true,
    include: ["src/**/*.test.{ts,tsx}"],
    transformMode: { web: [/\.[jt]sx?$/] },
  },
});
