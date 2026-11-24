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

- Split view: `splitViewStore.ts` + `components/SplitChatPane.tsx` (rendered by `routes/_chat.tsx`); opened from the right panel dropdown "Split view". Pane header: swap, remove left chat, close.
- Sidebar sorting (recent / alphabetical / manual, groups manual) and drag ordering live in `Cc3Sidebar.tsx`.
- Add-project palette: "New Repository" and "Connect Repository" (on this computer / clone) in `CommandPalette.tsx`.
- Composer notices (banners, question panel) are detached cards above the composer (`ChatView.tsx`, `ChatComposer.tsx`).

## Rules
See CLAUDE.md: work on the branch the user names; stay mergeable with upstream; never lose CC3 changes on merge.

## Current state
Initial CC3 port pushed to main. Not yet run as a built app: window chrome (frameless/transparent on Linux) is untested visually. Typecheck of touched files and DesktopWindow/appearanceFonts tests pass.

## Open
- Status strip and the permission chip (BranchToolbar) share one row under the composer (ChatView).
- Browser tests of RightPanelTabs/UsageLimitsPooled still expect tabs / "left".

## Next
- Launch `pnpm dev:desktop` and check window buttons, outline, resize, drag, fonts.
- Check Android build still resolves after iOS module removal (`apps/mobile`).
- Update workflows/docs that still mention T3 Code or removed platforms.

## Notes (2026-11-19, stability)
- Start menu entry "CC3" runs `~/.local/bin/cc3`: production build (`pnpm build:desktop` when sources are newer than `apps/server/dist/client/index.html`, then `pnpm start:desktop`), data via `T3CODE_HOME=~/.t3/cc3` (userdata symlinked to `~/.t3/dev`). Dev mode (`pnpm dev:desktop`) opens DevTools and relaunches Electron on every rebuild; use only for development, and stop leftover watchers.
- React Compiler bails out of a component when a closure reads a variable declared later (e.g. in ChatView). Then `useIncrementalTimelineEntries` loops ("Too many re-renders"). Check with a compiler logger on touched components.
- Installed app name is plain "CC3" (no "(Beta)").

## Notes (2026-11-20, UI round 3)
- Permissions control sits at the far right under the composer (ChatView, after BranchToolbar), not inside ComposerStatusStrip.
- Draft hero is a cycling greeting only (DraftHeroHeadline); project picker / triage hero removed.
- /mcp panel gets servers from the live context window, else from the Claude capability probe (`mcpServers` on the provider workspace snapshot, `q.mcpServerStatus()`).
- Right panel: single view (rightPanelStore.upsertSurface replaces); details card default hidden (DEFAULT_THREAD_PANEL_VISIBILITY.inlineOpen=false, storage v16); empty-state list has "Split view". 22 upstream multi-tab store tests are skipped on purpose.
- Split view: splitViewStore has focus + ratio; SplitChatPane has draggable divider with hover swap button; CSS in index.css dims the unfocused pane; sidebar click replaces the right chat when right is focused; sidebar context menu has "Open in split view".
- Settings nav: Projects and Labs pages hidden. `scripts/cc3-upstream-patches.sh` lists upstream fix/security commits for cherry-picking.
- PR and issue lists cover every repo of the gh account: PullRequestService.withAccountRepositories and IssueService.list add account repos (REST user/repos) borrowing a project as cwd carrier. Read-only for repos without a project.

## Wrapper system (2026-11-20)
Plan: docs/cc3-wrapper-plan.md. Goal: run real CLIs as TUIs under a PTY, structured events from their transcripts, GUI unchanged.
Done: WP1 wrapper core in apps/server/src/wrapper (PtyKeys, ScreenBuffer, TranscriptTail, WrapperSession, ClaudeTranscript parser, env scrub), unit tests, end-to-end smoke against real Claude Code (trust dialog, ready detection, paste submit, transcript events, turn end).
Next: WP2 `layerTuiQueryRunner` implementing ClaudeAgentSdkQuerySession (ClaudeAdapterV2.ts ~l.675) from ClaudeTranscriptEvent -> SDKMessage; WP3 approval bridge via hooks; WP4 slash-command popup showing PTY screen; WP5 Codex/OpenCode/Kilo profiles; WP6 per-instance `transport` switch (default sdk).
Done (WP2/WP3): apps/server/src/wrapper/ClaudeTuiQueryRunner.ts (layerTuiQueryRunner, implements ClaudeAgentSdkQueryRunner; PermissionRequest hook -> local HTTP bridge -> adapter canUseTool), ClaudeTuiMessageMapper.ts (transcript -> SDKMessage), claudeMcpList.ts. Enabled with env CC3_CLAUDE_TRANSPORT=tui (ProviderOrchestrationAdapterInfrastructure.ts), default stays SDK. Live test: CC3_TUI_LIVE=1 vp test run src/wrapper/ClaudeTuiQueryRunner.live.test.ts (prompt, Write permission via GUI callback, result: passes).
Known gaps in TUI transport: no token streaming, no fork/rollback, no context usage/MCP resource/tool calls, images in prompts dropped, setPermissionMode/applyFlagSettings no-ops, subagent lookups return null, no slash-command popup yet (WP4), no Codex/OpenCode profiles (WP5), env flag instead of per-instance setting (WP6).
Done (gap round): ScreenBuffer is now an @xterm/headless emulator (never shown to the user); local slash command output is read from the screen and sent to the GUI as a synthetic assistant message; images/PDFs via @path; fork via transcript copy (TranscriptFiles.ts, prompt uuid map sidecar); context usage approximated; live permission mode via Shift+Tab; effort/fast via slash commands; MCP tool/resource calls via McpClient.ts for host-registered servers; subagent lookups from subagent transcripts; per-instance ClaudeSettings.transport (sdk|tui) carried as CC3_CLAUDE_TRANSPORT env and dispatched in ClaudeQueryRunnerDispatch.ts.
Not done: token streaming in tui transport (CLI limit), Codex wrapper (codex not installed here). OpenCode/Kilo use their own server (kilo via OpenCode binaryPath, untested).
Done (CLI commands + real new chat): RPC provider.runCliCommand (contracts rpc.ts, ws.ts handler, client-runtime serverEnvironment.runCliCommand) -> ProviderInstance.runCliCommand (set by ClaudeDriver when transport=tui) -> wrapper/ClaudeCliCommands.ts (/mcp via `claude mcp list`; other commands on the chat's live TUI or a standby TUI via LiveTuiRegistry.ts, output read from the headless screen in ScreenCommand.ts). Web: ChatView.runLocalSlashCommand sends mcp/context/status/cost to the CLI (panel kind "cli" in Cc3CommandPanel), falls back to the CC3 screens when unsupported. Plus (scratch chats) creates a real thread (newChatLifecycle.ts materializeNewChat) and deletes it when left unused (onlyIfUnused). Start-up dialogs (trust, add-dir) answered by answerClaudeStartupDialog; ready-timeout error now carries the screen.
Open: Claude process still starts with the first message (adapter startTurn would need a warm-up path); first message takes ~15s in tui mode.

## Branches (2026-11-23)
- `cc3-GUI`: GUI-based Claude chat (plugin/MCP/config panels, `/cc3-config` runner). Frozen at b94f6e1f20.
- `main`: TUI experiment. Claude threads show the real Claude Code TUI in a terminal instead of the chat
  (`apps/web/src/components/ClaudeTuiPane.tsx`, hook in `ChatView.tsx` via `claudeTuiProps`). Drafts get a
  server thread through `threadEnvironment.create`. Server: `terminal/Manager.ts` `cc3ClaudeTuiShellCandidate`
  runs `claude --session-id|--resume <threadId> --settings {"theme":"custom:cc3"}` when runtimeEnv has `CC3_TUI_SESSION`.
- Theme file `~/.claude/themes/cc3.json` (only used via `--settings`, so only inside CC3).
- Font: `apps/web/public/fonts/CC3Mono-{Regular,Bold}.ttf (Red Hat Mono, OFL), default terminal font.

## TUI additions (2026-11-23)
- TUI launch script (`terminal/Manager.ts`): unsets inherited Claude markers, `CLAUDE_CODE_FORCE_SESSION_PERSISTENCE=1`, `--dangerously-skip-permissions` when client setting `claudeSkipPermissions` (default on, Settings > General), `xdg-open`/`BROWSER` shim prints OSC 777 `cc3-open;URL` which `ThreadTerminalDrawer` opens in the CC3 preview browser (live output only).
- Terminal title (OSC 0/2) from Claude renames the chat (`ClaudeTuiPane`); typed `/new` opens a new chat; thread details toggle hidden for TUI chats.
- Zoom: Ctrl +/-/0 in a terminal changes `fontSizeTerminal`; elsewhere `cc3Zoom.ts` (CSS zoom, localStorage).
- Open: bottom terminal "mirrors" Claude (cause not found), Clawd replacement not possible (rendered by the CLI).
- Drawer terminal no longer lists/reuses the `claude-tui` session (`state/terminalSessions.ts` filter; id in `types.ts`).
- Claude TUI gets the T3 MCP server (`--mcp-config` from `CC3_TUI_MCP_CONFIG`, issued in `terminal/cc3TuiMcp.ts`, wired in `ws.ts`) so preview_* browser tools work.
- Clawd in the welcome box is rewritten client-side (`replaceClaudeMascot`) to the CC3 logo letters: 27 private-use tiles U+E000.. in CC3 Mono (generator script scratch only).
- TUI MCP credentials are marked `terminal` (McpThreadCaller) so preview_* tools skip the active-run check; server key is "cc3"; cursor #e8e8e8.
- File panel: X closes file to root view, Suggest-changes button pastes `@path` into the claude-tui terminal, gutter + comments removed.
- Zoom: Ctrl +/-/0 outside terminals = Electron window zoom (`cc3Zoom.ts`, bridge `setWindowZoomFactor`); terminals divide their font by the window zoom (independent zoom).
- Header: smaller title, status dot (`claudeTuiStatus.ts`, from terminal title prefix), split right pane has no sidebar inset.
- Settings: pages chat/voice/providers/scheduled-tasks and default-model/text-generation rows hidden; new "Claude Code" section in General.
- TUI shims (`terminal/cc3TuiShims.ts`): `cc3-terminal` types a command into the drawer terminal (OSC cc3-term), `cc3-terminal-output` reads its history file; `--append-system-prompt` tells Claude about CC3.
