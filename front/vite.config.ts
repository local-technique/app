import { defineConfig, type Plugin } from "vite";
import vue from "@vitejs/plugin-vue";
import { configDefaults } from "vitest/config";

function routeChunkPreload(): Plugin {
  let base = "/";

  return {
    name: "route-chunk-preload",
    enforce: "post",
    configResolved(config) {
      base = config.base;
    },
    generateBundle(_, bundle) {
      const htmlAsset = Object.values(bundle).find(
        (v) => v.type === "asset" && v.fileName === "index.html",
      );
      if (!htmlAsset || typeof htmlAsset.source !== "string") return;

      const listingFormPages = [
        "/events/ListingPage.vue",
        "/incidents/ListingPage.vue",
        "/projects/ListingPage.vue",
        "/events/FormPage.vue",
        "/incidents/FormPage.vue",
        "/projects/FormPage.vue",
      ];
      const detailPages = [
        "/events/DetailPage.vue",
        "/incidents/DetailPage.vue",
        "/projects/DetailPage.vue",
      ];

      const earlyChunks: string[] = [];
      const lateChunks: string[] = [];

      for (const [, chunk] of Object.entries(bundle)) {
        if (chunk.type !== "chunk" || chunk.isEntry) continue;
        const facade = (chunk as any).facadeModuleId;
        if (typeof facade !== "string") continue;

        const normalized = facade.replace(/\\/g, "/");
        if (listingFormPages.some((p) => normalized.includes(p))) {
          earlyChunks.push(chunk.fileName);
        } else if (detailPages.some((p) => normalized.includes(p))) {
          lateChunks.push(chunk.fileName);
        }
      }

      const links = [
        ...earlyChunks.map((c) => `<link rel="modulepreload" crossorigin href="${base}${c}" />`),
        ...lateChunks.map((c) => `<link rel="prefetch" crossorigin href="${base}${c}" />`),
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
