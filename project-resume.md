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

- Added: CLI installer (host isPackaged=true, server at resources/app/apps/server), links.rs (single-instance + `t3code:` argv forwarding, queued until webLinks.setReady), launcher badge via gdbus (native::set_badge, cleared on focus), snapshots (DesktopSnapShot runs in the host with fake DesktopWindow/MacPermissions/ElectronDialog; workers bundled to host/dist/snapShot/*.cjs; stage.mjs builds native/kde-snap-shot + hyprland and copies gnome-extension), custom deb desktop template (Exec %U, MimeType t3code).
- Preview browser: bridge has no `preview`; web falls back to the server runtime (Playwright + Chrome on the host, screencast in ServerBrowserSurface). Not exercised in the UI.
- Verified by running the extracted .deb: snapshot state, CLI state, web-link forwarding from a second launch. Not verified: an actual capture, badge on a real launcher, preview UI.
- Caveat: CLI install relinks ~/.local/bin/t3 to the CC3 home; the original T3 app restores its own launcher on start. SIGTERM on the app leaves the server child running (only RunEvent::Exit cleans up).

- Window: frameless (decorations off, menu hidden); bridge.js draws min/max/close, polyfills navigator.windowControlsOverlay (so the web `.wco` layout applies) and starts native drags for `.drag-region`. Sidebar brand shows `CC3` when branding differs (SidebarChrome.tsx). Icon: two stacked Cs (icons/*).
- Preview sharpness: server stream quality default 92 and client requests >=2x frame size. Real desktop-runtime preview is impossible on WebKitGTK (no CDP, server drives pages over CDP); the server runtime with Chrome is used.
- RAM (idle, PSS sum): CC3 ~650-850 MB (server ~320, WebKit ~110-310, host ~100, shell ~80) vs Electron T3 ~350 MB. Server node uses far more native memory than Electron's; unexplained, next thing to investigate. User-local install: ~/.local/share/cc3-app + ~/.local/share/applications/CC3.desktop.

## Not implemented

Auto-update (state is "disabled"), AppImage bundle (only deb built), arm64, snapshot overlay animation, in-app `<webview>` preview runtime.

## Next ideas

- Updater against GitHub releases of corixien/cc3; AppImage; handle SIGTERM to stop the server child.
