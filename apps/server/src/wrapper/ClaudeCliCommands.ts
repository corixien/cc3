// @effect-diagnostics globalRandom:off cryptoRandomUUID:off cryptoRandomUUIDInEffect:off - session id for a standby TUI
// CC3 wrapper: runs a Claude Code slash command for a chat on the real CLI. A chat with a running
// wrapper session uses it; otherwise a standby TUI is started for the chat, so commands such as
// /context or /cost work before the first message. `/mcp` needs no TUI: `claude mcp list` is the
// CLI's own answer.
import * as Effect from "effect/Effect";
import * as Schema from "effect/Schema";
import type * as PtyAdapter from "@t3tools/shared/PtyAdapter";

import { listClaudeCliMcpServers } from "./claudeMcpList.ts";
import {
  findLiveTui,
  findStandbyTui,
  setStandbyTui,
  type LiveTuiHandle,
} from "./LiveTuiRegistry.ts";
import { runCommandOnScreen } from "./ScreenCommand.ts";
import { answerClaudeStartupDialog, isClaudeInputReady } from "./ScreenBuffer.ts";
import { scrubWrapperEnvironment } from "./wrapperEnvironment.ts";
import { TranscriptTail } from "./TranscriptTail.ts";
import { WrapperSession } from "./WrapperSession.ts";

export interface ClaudeCliCommandRequest {
  readonly cwd: string;
  readonly threadId?: string | undefined;
  readonly command: string;
}

export interface ClaudeCliCommandResult {
  readonly output: string;
  readonly mcpServers?: ReadonlyArray<{ readonly name: string; readonly status: string }>;
}

const READY_TIMEOUT_MS = 120_000;

export class ClaudeCliCommandError extends Schema.TaggedError<ClaudeCliCommandError>()(
  "ClaudeCliCommandError",
  { cause: Schema.Defect() },
) {
  override get message(): string {
    return "Claude could not run the command.";
  }
}

const fail = (cause: unknown) => new ClaudeCliCommandError({ cause });

export function makeClaudeCliCommandRunner(deps: {
  readonly pty: PtyAdapter.PtyAdapter["Service"];
  readonly binaryPath: string;
  readonly environment: NodeJS.ProcessEnv;
}) {
  const openStandby = Effect.fn("ClaudeCliCommands.openStandby")(function* (
    request: ClaudeCliCommandRequest,
    threadKey: string,
  ) {
    const environment = scrubWrapperEnvironment(deps.environment);
    const process = yield* deps.pty
      .spawn({
        shell: deps.binaryPath,
        args: ["--session-id", crypto.randomUUID()],
        cwd: request.cwd,
        cols: 120,
        rows: 40,
        env: environment,
      })
      .pipe(Effect.mapError(fail));
    const session = new WrapperSession({
      process,
      transcript: new TranscriptTail({ path: "/nonexistent", fromEnd: true }),
      isInputReady: isClaudeInputReady,
      answerStartupDialog: (screen) =>
        isClaudeTrustPrompt(screen) ? `${KEY_DOWN}${KEY_ENTER}` : undefined,
    });
    yield* Effect.tryPromise({
      try: () => session.waitUntilReady(READY_TIMEOUT_MS),
      catch: fail,
    }).pipe(Effect.tapError(() => Effect.sync(() => session.close())));
    const handle: LiveTuiHandle = {
      runCommand: (command) => runCommandOnScreen(session, command),
      isBusy: () => false,
      close: () => session.close(),
    };
    setStandbyTui(threadKey, handle);
    return handle;
  });

  return Effect.fn("ClaudeCliCommands.run")(function* (request: ClaudeCliCommandRequest) {
    if (/^\/mcp(\s|$)/i.test(request.command.trim())) {
      const servers = yield* Effect.tryPromise({
        try: () => listClaudeCliMcpServers(deps.binaryPath, request.cwd, deps.environment),
        catch: fail,
      });
      return {
        output: servers.map((server) => `${server.name}: ${server.status}`).join("\n"),
        mcpServers: servers,
      } satisfies ClaudeCliCommandResult;
    }
    const threadKey = request.threadId ?? `cwd:${request.cwd}`;
    const handle =
      findLiveTui(threadKey) ??
      findStandbyTui(threadKey) ??
      (yield* openStandby(request, threadKey));
    if (handle.isBusy()) {
      return {
        output: "A turn is running. Run the command again when it has finished.",
      } satisfies ClaudeCliCommandResult;
    }
    const output = yield* Effect.tryPromise({
      try: () => handle.runCommand(request.command.trim()),
      catch: fail,
    });
    return { output } satisfies ClaudeCliCommandResult;
  });
}
