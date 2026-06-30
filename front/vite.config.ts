import { defineConfig } from "vite";
import vue from "@vitejs/plugin-vue";
import { configDefaults } from "vitest/config";

export default defineConfig({
  plugins: [vue()],
  build: {
    rollupOptions: {
      output: {
        manualChunks(id: string) {
          if (id.includes("@scalar/api-reference")) {
            return "scalar";
          }
        },
      },
    },
  },
  test: {
    environment: "jsdom",
    setupFiles: [],
    globals: true,
    exclude: [...configDefaults.exclude, "tests/e2e/**"],
  },
});
