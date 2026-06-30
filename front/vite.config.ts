import { defineConfig, type Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
import { configDefaults } from "vitest/config";

function routeChunkPreload(): Plugin {
  return {
    name: "route-chunk-preload",
    enforce: "post",
    generateBundle(_, bundle) {
      const htmlAsset = Object.values(bundle).find(
        (v) => v.type === "asset" && v.fileName === "index.html",
      );
      if (!htmlAsset || typeof htmlAsset.source !== "string") return;

      const listingChunks: string[] = [];
      const otherChunks: string[] = [];

      for (const [, chunk] of Object.entries(bundle)) {
        if (chunk.type !== "chunk" || chunk.isEntry) continue;
        const fileName = chunk.fileName;
        if (!fileName.endsWith(".js")) continue;
        if (fileName.includes("scalar") || fileName.includes("Setting") || fileName.includes("ApiDoc")) continue;

        if (fileName.includes("ListingPage")) {
          listingChunks.push(fileName);
        } else if (fileName.includes("DetailPage") || fileName.includes("FormPage")) {
          otherChunks.push(fileName);
        }
      }

      const links = [
        ...listingChunks.map((c) => `<link rel="modulepreload" crossorigin href="/${c}" />`),
        ...otherChunks.map((c) => `<link rel="prefetch" crossorigin href="/${c}" />`),
      ].join("\n    ");

      if (!links) return;

      htmlAsset.source = (htmlAsset.source as string).replace(
        "</head>",
        `${links}\n  </head>`,
      );
    },
  };
}

export default defineConfig({
  plugins: [vue(), routeChunkPreload()],
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
