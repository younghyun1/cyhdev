/// <reference types="vitest" />
import { defineConfig } from "vite";
import solidPlugin from "@solidjs/vite-plugin";
import pkg from "./package.json" with { type: "json" };

export default defineConfig({
  plugins: [solidPlugin()],
  define: {
    __BUILD_TIMESTAMP__: JSON.stringify(new Date().toISOString()),
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
    transformMode: { web: [/\.[jt]sx?$/] },
  },
});
