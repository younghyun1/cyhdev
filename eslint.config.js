import js from "@eslint/js";
import tseslint from "typescript-eslint";
import solid from "eslint-plugin-solid/configs/typescript";

export default tseslint.config(
  js.configs.recommended,
  ...tseslint.configs.recommended,
  {
    files: ["src/**/*.{ts,tsx}"],
    ...solid,
    rules: {
      ...solid.rules,
      // Inverted under Solid 2.0: solid-js/store and solid-js/web no longer
      // exist (stores live in solid-js, DOM runtime in @solidjs/web). Drop
      // once eslint-plugin-solid ships 2.0-aware rules.
      "solid/imports": "off",
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
