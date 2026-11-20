# CC3 wrapper system plan

Goal: replace the SDK/protocol-driven provider transports with wrappers around the real CLIs
(Claude Code, Codex, OpenCode, Kilo Code, ...) running as their interactive TUI in a PTY, while the
GUI, orchestration and event model stay unchanged. Every CLI command, config file, MCP server,
plugin, hook and slash command then behaves exactly as in the CLI, because it is the CLI.

Reference: Orca IDE (`stablyai/orca`, AppImage in `~/Applications`) runs agents in `node-pty`
terminals and puts its UI around them. CC3 differs in keeping a structured chat GUI, so the PTY
needs a structured side channel.

## Architecture

```
GUI (unchanged)  <-  orchestration-v2 events (unchanged)  <-  Adapter (unchanged)
                                                                  |
                                                       transport seam (per provider)
                                                                  |
                                              WrapperSession  (new, apps/server/src/wrapper/)
                                              |- PtyHost        real CLI TUI in node-pty (existing terminal NodePtyAdapter)
                                              |- InputDriver    prompt paste + Enter, Esc interrupt, slash commands
                                              |- EventSource    structured stream read from the CLI's own files/APIs
                                              |- ApprovalBridge permission requests via CLI hooks -> server callback
                                              '- ScreenTap      raw PTY stream, shown in a terminal popup for slash commands
```

The transport seam already exists per provider. Claude: `ClaudeAgentSdkQueryRunner` (a Context
service, `ClaudeAdapterV2.ts`) yields a `ClaudeAgentSdkQuerySession` (`messages`, `offer`,
`interrupt`, `setModel`, ...). A `layerTuiQueryRunner` implementing it replaces the SDK layer, so
`ClaudeAdapterV2` and everything above it stay as they are. Selection is a per-instance setting
(`transport: "sdk" | "tui"`), default `sdk` until the wrapper is proven.

## Per-CLI event sources

| CLI | Structured source | Input | Approvals |
| --- | --- | --- | --- |
| Claude Code | `~/.claude/projects/<cwd>/<session>.jsonl` (tail), `--session-id` / `--resume` | bracketed paste + Enter | `PermissionRequest`/`PreToolUse` hook injected via `--settings`, HTTP callback |
| Codex | `~/.codex/sessions/**/rollout-*.jsonl` (tail) | paste + Enter | `notify`/hooks config, else keep app-server |
| OpenCode, Kilo Code | their local HTTP server + SSE (`opencode serve`, TUI `attach`) | server API | server permission API |
| ACP agents (Cursor, Grok, Kiro, Pi, Muse, ...) | unchanged (ACP is already a wrapper protocol) | | |

## Spike results (2026-11-20, Claude Code 2.1.304)

- The TUI runs headless in `node-pty`; a prompt pasted as `ESC[200~ ... ESC[201~` and confirmed
  with Enter is processed, and the transcript JSONL receives `user`, `assistant` (whole messages,
  `stop_reason`), `system` (`turn_duration`), `attachment`, `permission-mode`, `cost-state` lines.
- The environment must be scrubbed of `CLAUDE_CODE_*`, `CLAUDECODE` and `CLAUDE_AGENT_*`
  variables, or the CLI turns transcript saving off ("inherited CLAUDE_CODE_CHILD_SESSION").
- A folder the CLI has not seen shows the trust dialog; the wrapper must pre-accept it
  (`projects[cwd].hasTrustDialogAccepted` in `~/.claude.json`) or answer it.
- The TUI needs about 10-15 seconds before it accepts input in a user environment with hooks;
  readiness is detected from the prompt (`❯`) on the PTY screen, not a timer.
- No token deltas: the transcript holds whole messages. Streaming text is lost in this transport
  unless the screen is parsed; accepted degradation.

## Work packages

1. **Wrapper core** (`apps/server/src/wrapper/`): `PtyHost` (spawn, resize, screen buffer,
   readiness probe), `TranscriptTail` (inode-safe tail with rotation/resume), env scrub, trust
   pre-accept. Unit tests with a fake PTY.
2. **Claude profile**: JSONL line -> `SDKMessage` mapper (`system:init`, `assistant`, `user`
   tool results, `result` with usage), `TuiQuerySession` (`offer` paste, `interrupt` Esc,
   `setModel` -> `/model`, `setPermissionMode` -> Shift+Tab cycling or restart with flag,
   `mcpServerStatus` -> `claude mcp list`), `layerTuiQueryRunner`.
3. **Approval bridge**: local HTTP endpoint on the server, hook command injected per session,
   maps hook payloads to the adapter's `canUseTool` callback and back.
4. **Slash commands**: commands typed in the composer are forwarded to the PTY; the GUI popup
   shows the PTY screen (xterm) for commands whose output is not in the transcript (`/mcp`,
   `/model`, `/config`).
5. **Codex / OpenCode / Kilo Code profiles** on the same core.
6. **Switch**: instance setting `transport`, migration of defaults, removal of the SDK path only
   after parity checks (resume, fork, rollback, background tasks, subagents).

## Risks

- Upstream merges: all new code lives in new files; the only upstream edits are the layer
  selection and the driver setting.
- Parity gaps of the TUI transport: no streaming deltas, rollback/fork via the CLI's own
  `--resume`/`--fork-session`, background task control via TUI only.
- Versioned transcript formats: the mapper is isolated and covered by fixture tests recorded from
  real sessions.
