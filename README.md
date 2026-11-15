# CC3 - Corixien Code 3

Linux desktop app for the [T3 Code](https://github.com/pingdotgg/t3code) control plane, built on
Tauri (Rust) instead of Electron. The web UI (`apps/web`, React + Tailwind) and the server
(`apps/server`) are unchanged, so CC3 stays compatible with the web app and the mobile apps.
Forked from t3code (MIT); the upstream README is kept in `docs/README.t3code.md`.

## How it works

| Piece           | Where                        | Role                                                                                                                                     |
| --------------- | ---------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------- |
| Tauri shell     | `apps/cc3-desktop/src-tauri` | Window, menu, dialogs, starts the T3 server in desktop mode, injects `window.desktopBridge`                                              |
| Bridge          | `src-tauri/src/bridge.js`    | Implements the `DesktopBridge` contract (`packages/contracts`) on Tauri IPC                                                              |
| Desktop host    | `apps/cc3-desktop/host`      | Node sidecar reusing `apps/desktop` logic (SSH environments, settings, connection catalog, server exposure, Tailscale, provider sign-in) |
| Server + web UI | `apps/server`, `apps/web`    | Unchanged T3 Code                                                                                                                        |

`apps/desktop` is the old Electron shell, kept as the reference for the remaining features.

## Run from source

```bash
pnpm install
pnpm vp run --filter @t3tools/web --filter t3 build
cd apps/cc3-desktop && pnpm tauri dev
```

Data lives in `~/.cc3` (override with `CC3_HOME`; point it at `~/.t3` to share data with the
Electron app, but not while that app is running: one server per data directory).

## Package (.deb)

```bash
pnpm vp run --filter @t3tools/web --filter t3 build
cd apps/cc3-desktop && pnpm exec vp pack && node scripts/stage.mjs && pnpm tauri build --bundles deb
```

## Status

Works: window and native menu, folder/file pickers, context menus, client settings, saved
connections, SSH environments, network exposure and Tailscale Serve, provider sign-in callback,
zoom, fullscreen, theme.

Not yet: embedded preview browser (needs Electron `<webview>`; the bridge omits `preview`, so the
UI disables it), screen snapshots, auto-update, `t3` CLI installer, `t3://` deep links.

Debug switches: `CC3_DEBUG=1` (log bridge calls), `CC3_DEVTOOLS=1` (open devtools).
