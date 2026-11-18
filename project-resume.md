# CC3 project resume

## What this is
CC3 = fork of upstream t3code (pingdotgg/t3code, Electron + web + Node server + Expo mobile). Only the Linux desktop app and the Android app are kept.

## Repo layout
- `origin` = github.com/corixien/cc3, `upstream` = github.com/pingdotgg/t3code.
- Branches on origin: `main` (this Electron fork), `cc3-tauri` (old Tauri implementation), `cc3-performance` (old native Blitz UI). The last two are archived, not developed further.
- `apps/desktop` Electron shell, `apps/web` UI, `apps/server` backend, `apps/mobile` Expo/Android, `packages/*` shared.

## CC3 changes on top of upstream
- Removed: apps/marketing, iOS native modules, dmg packaging, macOS/Windows workflows and dist scripts, iOS/macOS/Windows icon files.
- Branding: "T3 Code" replaced by "CC3" in source; sidebar wordmark is the text "CC3".
- Logo: all icons under `assets/{dev,nightly,prod}`, `apps/web/public`, `apps/mobile/assets/android-*` use the CC3 logo with outline (source: cc3-tauri `icons/icon.png`).
- Linux window: frameless + transparent, rounded 14px with 1px outline, own minimize/maximize/close buttons top right, Window Controls Overlay polyfill. Files: `apps/desktop/src/cc3WindowChrome.ts` (preload), `cc3WindowControls.ts` (main IPC), small hooks in `preload.ts`, `main.ts`, `window/DesktopWindow.ts`.
- Font: Lexend Deca for the interface and terminal default (`apps/web/public/fonts`, `index.css`, `appearanceFonts.ts`, `terminal/ghostty/surface.ts`). Code keeps a monospace font.

- Sidebar: `apps/web/src/components/Cc3Sidebar.tsx` replaces the project-grouped upstream sidebar (mounted in `AppSidebarLayout.tsx`): flat chat list, title + working/needs-input marker, user folders (localStorage `cc3:sidebar-folders`), drag or context menu to sort, no settle UI. Upstream `Sidebar.tsx` is untouched but unused.
- "Without a project" / "+" = Home: `ManagedProjectFolders.ensureScratchProject` opens the user's home directory as project "Home".
- Composer status strip: `components/chat/ComposerStatusStrip.tsx` (context, repo/branch, directory, usage used-%). Usage UI shows share used, not left.
- Right panel: single view with dropdown switcher (`CC3_SINGLE_VIEW` in `RightPanelTabs.tsx`). Stage artwork disabled in `SidebarStageBackdrop.tsx`.

## Rules
See CLAUDE.md: work on the branch the user names; stay mergeable with upstream; never lose CC3 changes on merge.

## Current state
Initial CC3 port pushed to main. Not yet run as a built app: window chrome (frameless/transparent on Linux) is untested visually. Typecheck of touched files and DesktopWindow/appearanceFonts tests pass.

## Open
- Permission ("Full access") chip is still in the composer controls, not in the status strip.
- Browser tests of RightPanelTabs/UsageLimitsPooled still expect tabs / "left".

## Next
- Launch `pnpm dev:desktop` and check window buttons, outline, resize, drag, fonts.
- Check Android build still resolves after iOS module removal (`apps/mobile`).
- Update workflows/docs that still mention T3 Code or removed platforms.
