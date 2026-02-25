import { defineConfig } from "vitest/config";
import { resolve } from "path";
import dts from "vite-plugin-dts";

export default defineConfig({
  plugins: [dts()],
  build: {
    lib: {
      entry: resolve(__dirname, "src/lib.ts"),
      name: "AnyStore",
      fileName: (format) => `lib.${format}.js`,
      formats: ["es"],
    },
    rollupOptions: {
      external: [], // Add external dependencies here if needed
      output: {
        globals: {
          // Map external dependencies to global variables for UMD build
        },
      },
    },
  },
  server: {
    headers: {
      "Cross-Origin-Opener-Policy": "same-origin",
      "Cross-Origin-Embedder-Policy": "require-corp",
    },
    fs: {
      // Allow serving files from one level up to the project root
      allow: [".."],
      // Or specify exact paths
      // allow: ['/path/to/external/dir', '../other-project']
    },
  },
  test: {
    environment: "node",
    benchmark: {},
    include: ["tests/**/*.test.ts"],
  },
});
