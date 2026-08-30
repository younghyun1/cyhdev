import js from "@eslint/js";
import tseslint from "typescript-eslint";
import solid from "eslint-plugin-solid/configs/v2";

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    ...solid,
    rules: {
      ...solid.rules,
      // The 0.16 v2 map points JSX at solid-js, but the RC exports it from
      // @solidjs/web. Keep valid type imports until the rule catches up.
      "solid/imports": "off",
      // Solid's compiler assigns bare `let` refs from JSX. ESLint cannot see
      // those writes and reports every ref as permanently undefined.
      "no-unassigned-vars": "off",
      "@typescript-eslint/no-unused-vars": [
        "warn",
        { argsIgnorePattern: "^_", varsIgnorePattern: "^_" },
      ],
      "@typescript-eslint/consistent-type-imports": "warn",
      "@typescript-eslint/no-empty-object-type": "off",
    },
  },
  {
    ignores: ["dist/", "node_modules/"],
  },
);
