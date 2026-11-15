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

M2 done: server runs in desktop mode (bootstrap JSON over stdin, `--bootstrap-fd 0`); Rust waits for `GET / == 200`, opens window with init script (src-tauri/src/bridge.js) that defines window.desktopBridge. Native ops in src-tauri/src/native.rs (`cc3_native` command, ACL via build.rs app manifest + capabilities/default.json, required for the remote http origin). Native app menu, DOM context menu, file-backed client settings/connection catalog, folder/file dialogs, openExternal, theme, zoom, fullscreen, readMedia work. Bearer token: same-origin form POST /oauth/token in bridge.js.
Debug: CC3_DEBUG=1 (renderer log via local HTTP listener, bridge calls logged), CC3_DEVTOOLS=1 (open devtools).
Not implemented (stubbed in bridge.js, typed defaults/rejects): SSH environments, server exposure/tailscale, CLI command install, updater, WSL (n/a on Linux), snapshot, provider auth callback, probeRemoteEditors, embedded preview browser (`preview` intentionally absent so web disables it).

## Next

- Node host (or Rust) for SSH/exposure/tailscale/cli/updater using packages/ssh etc.
- Packaging: bundle node+server as resources, deb/AppImage.
- Preview browser via Tauri child webviews (WebKitGTK limits).
