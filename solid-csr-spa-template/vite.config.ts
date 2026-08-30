/// <reference types="vitest" />
import { defineConfig } from "vite";
import solidPlugin from "vite-plugin-solid";
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
  },
  test: {
    environment: "jsdom",
    globals: true,
    transformMode: { web: [/\.[jt]sx?$/] },
  },
});
