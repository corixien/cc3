import * as NodePath from "node:path";
import { defineConfig } from "vite-plus";

// Bundles host/main.ts into one file; the keyring addon and undici stay external.
export default defineConfig({
  pack: {
    entry: ["host/main.ts"],
    outDir: "host/dist",
    clean: true,
    format: "esm",
    platform: "node",
    alias: { electron: NodePath.join(import.meta.dirname, "host/electron-inert.mjs") },
    deps: { neverBundle: [/^@napi-rs\/keyring/, /^undici$/], alwaysBundle: [/.*/] },
  },
});
