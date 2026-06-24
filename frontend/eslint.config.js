import js from "@eslint/js";
import globals from "globals";
import pluginVue from "eslint-plugin-vue";
import ts from "typescript-eslint";

// noc-lens 前端 ESLint（flat config）
export default [
  { ignores: ["dist/**", "node_modules/**"] },
  js.configs.recommended,
  ...ts.configs.recommended,
  ...pluginVue.configs["flat/essential"],
  {
    files: ["**/*.{ts,vue}"],
    languageOptions: {
      globals: { ...globals.browser },
      parserOptions: { parser: ts.parser },
    },
    rules: {
      "@typescript-eslint/no-explicit-any": "off",
      "@typescript-eslint/no-unused-vars": ["warn", { argsIgnorePattern: "^_" }],
      "vue/multi-word-component-names": "off",
    },
  },
];
