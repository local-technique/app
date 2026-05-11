import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { configDefaults } from "vitest/config";

export default defineConfig({
  plugins: [vue()],
  test: {
    environment: "jsdom",
    setupFiles: [],
    globals: true,
    exclude: [...configDefaults.exclude, "tests/e2e/**"],
  },
});
