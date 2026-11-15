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

Working (verified by running the app, incl. the extracted .deb with bundled node/server/host):

- Tauri shell (src-tauri/src): server lifecycle (desktop mode, stdin bootstrap, same port/token on restart), native menu, `cc3_native` (dialogs, openExternal, theme, zoom, readMedia), `cc3_host` RPC, paths.rs (bundled vs dev paths).
- bridge.js defines window.desktopBridge; DOM context menu; same-origin bearer exchange.
- Node host (apps/cc3-desktop/host/main.ts) registers existing apps/desktop IPC methods against a fake ipcMain over stdio: SSH environments, client settings, connection catalog (keyring-based safeStorage replacement in host/safeStorage.ts), server exposure + Tailscale (restarts server via cc3:relaunch-backend), CLI command, provider sign-in callback. `electron` is stubbed (host/electron-stub.mjs loader in dev, host/electron-inert.mjs when bundled).
- Packaging: `vp pack` (host bundle) -> `node scripts/stage.mjs` -> `pnpm tauri build --bundles deb` (240 MB).
- Upstream pre-commit hooks (fmt, lint, knip) run on commit; fix lint instead of bypassing.

## Not implemented

Embedded preview browser (bridge omits `preview`), snapshots, auto-update (state is "disabled"), CLI installer reports supported:false, t3:// deep links, notification badge, AppImage bundle (only deb built), arm64.

## Next ideas

- Preview browser via Tauri child webviews; updater against GitHub releases of corixien/cc3; deep-link plugin for webLinks; AppImage.
