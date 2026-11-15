# CC3 (Corixien Code 3) - project resume

Goal: Linux desktop app = Tauri (Rust) shell replacing the Electron shell of t3code. Web UI (apps/web, React+Tailwind) and server (apps/server) are reused unchanged and stay compatible with web/iOS/Android. Forked from pingdotgg/t3code (MIT), no further merging with upstream. Repo: github.com/corixien/cc3 (public).

## Layout

- apps/cc3-desktop: Tauri app (src-tauri/src/main.rs). apps/desktop = old Electron shell, kept as porting reference (131-method window.desktopBridge in src/preload.ts).

## Run

1. pnpm install
2. pnpm vp run --filter @t3tools/web --filter t3 build (builds apps/server/dist incl. bundled client)
3. cd apps/cc3-desktop && pnpm tauri dev (or cargo build in src-tauri, run target/debug/cc3)
   Env: CC3_HOME (data dir, default ~/.cc3), CC3_SERVER_ENTRY, CC3_NODE.

## Current state

M1 done: Rust spawns `node server/dist/bin.mjs --no-browser --port <free> --base-dir ~/.cc3`, parses `pairingUrl:` from stdout, opens window on it, kills server on exit. UI loads and works.

## Next (target: 1:1 with Electron bridge)

- Inject window.desktopBridge (Tauri IPC) so desktop-only UI works: folder picker, context menu, app menu, openExternal, settings, badge, zoom, fullscreen state.
- Use desktop mode + bootstrap (stdin/fd3, bearer token) instead of web-mode pairing URL.
- Port logic from apps/desktop/src: backend manager, SSH, connection catalog, Tailscale/server exposure, updater, linux URL handler, CLI shim, snapshot, embedded preview browser (hard in WebKitGTK).
- Packaging: bundle node/server as resources, deb/AppImage via tauri build.
  Risks: WebKitGTK vs Chromium rendering/perf; preview browser tabs.
