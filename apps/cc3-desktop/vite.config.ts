import * as NodePath from "node:path";
import { defineConfig } from "vite-plus";

const alias = { electron: NodePath.join(import.meta.dirname, "host/electron-inert.mjs") };

// Bundles host/main.ts into one file; the keyring addon and undici stay external.
// The snapshot workers are separate CommonJS files that DesktopSnapShot spawns by path.
export default defineConfig({
  pack: [
    {
      entry: ["host/main.ts"],
      outDir: "host/dist",
      clean: true,
      format: "esm",
      platform: "node",
      alias,
      deps: { neverBundle: [/^@napi-rs\/keyring/, /^undici$/], alwaysBundle: [/.*/] },
    },
    {
      entry: [
        "../desktop/src/snapShot/GlobalShiftShortcutWorker.ts",
        "../desktop/src/snapShot/RegionSnapShotWorker.ts",
        "../desktop/src/snapShot/SnapShotAccessibilityWorker.ts",
      ],
      outDir: "host/dist/snapShot",
      clean: false,
      format: "cjs",
      platform: "node",
      outExtensions: () => ({ js: ".cjs" }),
      alias,
      deps: { alwaysBundle: (id: string) => !id.startsWith("node:") },
    },
  ],
});
