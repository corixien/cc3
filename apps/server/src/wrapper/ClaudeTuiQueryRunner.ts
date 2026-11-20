// @effect-diagnostics nodeBuiltinImport:off globalDate:off globalTimers:off globalRandom:off cryptoRandomUUID:off cryptoRandomUUIDInEffect:off globalTimersInEffect:off effectFnOpportunity:off - the TUI session is promise-based and talks to a PTY, a local HTTP server and plain files
// CC3 wrapper: runs the real Claude Code TUI under a PTY and presents it to `ClaudeAdapterV2` as a
// `ClaudeAgentSdkQuerySession`. Every CLI feature (slash commands, MCP, plugins, hooks, config)
// behaves as in the CLI because it is the CLI; structured events come from its transcript file and
// permission prompts reach the GUI through a `PermissionRequest` hook that calls back here.
import * as NodeChildProcess from "node:child_process";
import * as NodeFs from "node:fs";
import * as NodeHttp from "node:http";
import * as NodeOs from "node:os";
import * as NodeUtil from "node:util";

import type { CanUseTool, PermissionResult, SDKMessage } from "@anthropic-ai/claude-agent-sdk";
import * as Effect from "effect/Effect";
import * as Layer from "effect/Layer";
import * as SecureRandom from "effect/crypto/SecureRandom";
import * as Stream from "effect/Stream";
import * as PtyAdapter from "@t3tools/shared/PtyAdapter";
import * as NodePtyAdapter from "../terminal/NodePtyAdapter.ts";

import {
  ClaudeAgentSdkQueryRunner,
  ClaudeAgentSdkQueryRunnerError,
  type ClaudeAgentSdkQueryOpenInput,
  type ClaudeAgentSdkQuerySession,
} from "../orchestration-v2/Adapters/ClaudeAdapterV2.ts";
import { claudeTranscriptPath, parseClaudeTranscriptLine } from "./ClaudeTranscript.ts";
import { ClaudeTuiMessageMapper, claudeTuiInitMessage } from "./ClaudeTuiMessageMapper.ts";
import { listClaudeCliMcpServers } from "./claudeMcpList.ts";
import { isClaudeInputReady, isClaudeTrustPrompt } from "./ScreenBuffer.ts";
import { KEY_DOWN, KEY_ENTER } from "./ptyKeys.ts";
import { scrubWrapperEnvironment } from "./wrapperEnvironment.ts";
import { WrapperSession } from "./WrapperSession.ts";
import { TranscriptTail } from "./TranscriptTail.ts";

const READY_TIMEOUT_MS = 120_000;
const POLL_INTERVAL_MS = 250;

const execFile = NodeUtil.promisify(NodeChildProcess.execFile);

const runnerError = (method: string) => (cause: unknown) =>
  new ClaudeAgentSdkQueryRunnerError({ method, cause });

type Json = Record<string, unknown>;
const isRecord = (value: unknown): value is Json =>
  typeof value === "object" && value !== null && !Array.isArray(value);

/** Text of a prompt; attachments other than text are not representable in the TUI input box. */
function promptText(content: unknown): string {
  if (typeof content === "string") return content;
  if (!Array.isArray(content)) return "";
  return content
    .map((block) => (isRecord(block) && block.type === "text" ? String(block.text ?? "") : ""))
    .filter((text) => text.length > 0)
    .join("\n");
}

function readSettingsObject(settings: unknown): Json {
  if (isRecord(settings)) return settings;
  if (typeof settings !== "string") return {};
  try {
    const parsed: unknown = JSON.parse(
      settings.trimStart().startsWith("{") ? settings : NodeFs.readFileSync(settings, "utf8"),
    );
    return isRecord(parsed) ? parsed : {};
  } catch {
    return {};
  }
}

interface PermissionBridge {
  readonly port: number;
  readonly close: () => void;
}

/** Local endpoint the `PermissionRequest` hook posts to; it answers with the GUI's decision. */
function startPermissionBridge(input: {
  readonly canUseTool: CanUseTool;
  readonly resolveToolUseId: (toolName: string, toolInput: unknown) => Promise<string>;
}): Promise<PermissionBridge> {
  return new Promise((resolve, reject) => {
    const server = NodeHttp.createServer((request, response) => {
      const abort = new AbortController();
      response.on("close", () => abort.abort());
      let body = "";
      request.on("data", (chunk: Buffer) => (body += chunk.toString("utf8")));
      request.on("end", () => {
        void (async () => {
          let decision: PermissionResult | null;
          try {
            const payload: unknown = JSON.parse(body);
            const toolName = isRecord(payload) ? String(payload.tool_name ?? "") : "";
            const toolInput =
              isRecord(payload) && isRecord(payload.tool_input) ? payload.tool_input : {};
            const toolUseID = await input.resolveToolUseId(toolName, toolInput);
            decision = await input.canUseTool(toolName, toolInput, {
              signal: abort.signal,
              toolUseID,
            } as Parameters<CanUseTool>[2]);
          } catch (cause) {
            decision = { behavior: "deny", message: `CC3 could not decide: ${String(cause)}` };
          }
          response.setHeader("content-type", "application/json");
          // No decision leaves the prompt to the TUI itself.
          if (decision === null) return response.end("{}");
          response.end(
            JSON.stringify({
              hookSpecificOutput: {
                hookEventName: "PermissionRequest",
                decision:
                  decision.behavior === "allow"
                    ? {
                        behavior: "allow",
                        ...(decision.updatedInput ? { updatedInput: decision.updatedInput } : {}),
                        ...(decision.updatedPermissions
                          ? { updatedPermissions: decision.updatedPermissions }
                          : {}),
                      }
                    : {
                        behavior: "deny",
                        message: decision.message,
                        ...(decision.interrupt ? { interrupt: true } : {}),
                      },
              },
            }),
          );
        })();
      });
    });
    server.once("error", reject);
    server.listen(0, "127.0.0.1", () => {
      const address = server.address();
      if (address === null || typeof address === "string") return reject(new Error("no port"));
      resolve({ port: address.port, close: () => server.close() });
    });
  });
}

function buildClaudeArguments(input: {
  readonly options: ClaudeAgentSdkQueryOpenInput["options"];
  readonly sessionId: string;
  readonly resume: boolean;
  readonly bridgePort: number | undefined;
}): string[] {
  const { options } = input;
  const args: string[] = input.resume
    ? ["--resume", input.sessionId]
    : ["--session-id", input.sessionId];
  args.push("--model", options.model);
  if (options.permissionMode === "bypassPermissions") args.push("--dangerously-skip-permissions");
  else args.push("--permission-mode", options.permissionMode);
  if ("effort" in options && typeof options.effort === "string")
    args.push("--effort", options.effort);
  if (Array.isArray(options.tools)) args.push("--tools", options.tools.join(","));
  if (options.allowedTools?.length) args.push("--allowedTools", options.allowedTools.join(","));
  if (options.disallowedTools?.length)
    args.push("--disallowedTools", options.disallowedTools.join(","));
  for (const directory of options.additionalDirectories ?? []) args.push("--add-dir", directory);
  const systemPrompt = options.systemPrompt;
  if (isRecord(systemPrompt) && systemPrompt.type === "preset" && systemPrompt.append)
    args.push("--append-system-prompt", String(systemPrompt.append));

  const settings = readSettingsObject(options.settings);
  if (input.bridgePort !== undefined) {
    const hook = {
      type: "command",
      command: `curl -sS --max-time 86400 -H 'Content-Type: application/json' --data-binary @- http://127.0.0.1:${input.bridgePort}/permission`,
    };
    const hooks = isRecord(settings.hooks) ? settings.hooks : {};
    const existing = Array.isArray(hooks.PermissionRequest) ? hooks.PermissionRequest : [];
    settings.hooks = { ...hooks, PermissionRequest: [...existing, { hooks: [hook] }] };
  }
  if (Object.keys(settings).length > 0) args.push("--settings", JSON.stringify(settings));

  const servers = Object.fromEntries(
    Object.entries(options.mcpServers ?? {}).filter(
      ([, server]) => isRecord(server) && server.type !== "sdk",
    ),
  );
  if (Object.keys(servers).length > 0)
    args.push("--mcp-config", JSON.stringify({ mcpServers: servers }));

  for (const [flag, value] of Object.entries(options.extraArgs ?? {})) {
    args.push(`--${flag}`);
    if (typeof value === "string") args.push(value);
  }
  return args;
}

class AsyncMessageQueue {
  private readonly items: SDKMessage[] = [];
  private waiting: (() => void) | undefined;
  private closed = false;

  push(message: SDKMessage): void {
    this.items.push(message);
    this.waiting?.();
  }

  close(): void {
    this.closed = true;
    this.waiting?.();
  }

  async *iterate(): AsyncGenerator<SDKMessage> {
    while (true) {
      const next = this.items.shift();
      if (next !== undefined) {
        yield next;
        continue;
      }
      if (this.closed) return;
      await new Promise<void>((resolve) => {
        this.waiting = resolve;
      });
      this.waiting = undefined;
    }
  }
}

const unsupported = (feature: string) =>
  Effect.fail(
    new ClaudeAgentSdkQueryRunnerError({
      method: feature,
      cause: new Error(`${feature} is not available while Claude runs as a terminal wrapper.`),
    }),
  );

const layerTuiQueryRunnerWithPty = Layer.effect(
  ClaudeAgentSdkQueryRunner,
  Effect.gen(function* () {
    const pty = yield* PtyAdapter.PtyAdapter;
    const random = yield* SecureRandom.SecureRandom;

    const open = Effect.fn("ClaudeTuiQueryRunner.open")(function* (
      input: ClaudeAgentSdkQueryOpenInput,
    ) {
      const { options } = input;
      const sessionId = options.sessionId ?? options.resume ?? (yield* random.uuidV4());
      const resume = options.resume !== undefined || input.resume === true;
      const cwd = options.cwd ?? process.cwd();
      const environment = scrubWrapperEnvironment({
        ...process.env,
        ...(options.env as NodeJS.ProcessEnv | undefined),
      });
      const executable = options.pathToClaudeCodeExecutable ?? "claude";
      const configDir = environment.CLAUDE_CONFIG_DIR ?? `${NodeOs.homedir()}/.claude`;

      const queue = new AsyncMessageQueue();
      const mapper = new ClaudeTuiMessageMapper(sessionId, () => crypto.randomUUID());
      let session: WrapperSession | undefined;
      let pump: ReturnType<typeof setInterval> | undefined;
      const drain = () => {
        for (const line of session?.pollTranscript() ?? [])
          for (const event of parseClaudeTranscriptLine(line))
            for (const message of mapper.map(event)) queue.push(message);
      };

      const bridge =
        options.canUseTool !== undefined && options.permissionMode !== "bypassPermissions"
          ? yield* Effect.tryPromise({
              try: () =>
                startPermissionBridge({
                  canUseTool: options.canUseTool as CanUseTool,
                  resolveToolUseId: async (toolName) => {
                    // The transcript line holding the tool call may land just after the hook fires.
                    for (let attempt = 0; attempt < 8; attempt++) {
                      drain();
                      const match = mapper.pendingToolUses.findLast((use) => use.name === toolName);
                      if (match) return match.id;
                      await new Promise((resolve) => setTimeout(resolve, 100));
                    }
                    return crypto.randomUUID();
                  },
                }),
              catch: runnerError("permissionBridge"),
            })
          : undefined;

      const args = buildClaudeArguments({
        options,
        sessionId,
        resume,
        bridgePort: bridge?.port,
      });
      const process_ = yield* pty
        .spawn({ shell: executable, args, cwd, cols: 120, rows: 40, env: environment })
        .pipe(Effect.mapError(runnerError("spawn")));
      session = new WrapperSession({
        process: process_,
        transcript: new TranscriptTail({
          path: claudeTranscriptPath({ configDir, cwd, sessionId }),
          fromEnd: resume,
        }),
        isInputReady: isClaudeInputReady,
        // Highlight starts on "No, exit": move down to "Yes, I trust this folder".
        answerStartupDialog: (screen) =>
          isClaudeTrustPrompt(screen) ? `${KEY_DOWN}${KEY_ENTER}` : undefined,
      });
      const live = session;
      const shutdown = () => {
        if (pump !== undefined) clearInterval(pump);
        bridge?.close();
        live.close();
        queue.close();
      };

      yield* Effect.tryPromise({
        try: () => live.waitUntilReady(READY_TIMEOUT_MS),
        catch: runnerError("ready"),
      }).pipe(Effect.tapError(() => Effect.sync(shutdown)));

      const version = yield* Effect.tryPromise({
        try: () => execFile(executable, ["--version"], { env: environment }),
        catch: runnerError("version"),
      }).pipe(
        Effect.map((result) => result.stdout.trim().split(/\s+/)[0] ?? "unknown"),
        Effect.orElseSucceed(() => "unknown"),
      );
      queue.push(
        claudeTuiInitMessage(
          {
            sessionId,
            cwd,
            model: options.model,
            permissionMode: options.permissionMode,
            version,
          },
          crypto.randomUUID(),
        ),
      );
      pump = setInterval(() => {
        drain();
        if (live.hasExited) shutdown();
      }, POLL_INTERVAL_MS);

      const sessionApi: ClaudeAgentSdkQuerySession = {
        messages: Stream.fromAsyncIterable(queue.iterate(), runnerError("messages")),
        offer: (message, onQueued) =>
          Effect.tryPromise({
            try: async () => {
              const text = promptText(message.message.content);
              if (message.uuid) mapper.expectPrompt(message.uuid);
              await live.submit(text);
              onQueued?.();
            },
            catch: runnerError("offer"),
          }),
        setModel: (model) =>
          Effect.tryPromise({
            try: () => live.submit(`/model ${model}`),
            catch: runnerError("setModel"),
          }),
        // Flag settings and permission mode apply at launch; the TUI keeps its own live state.
        applyFlagSettings: () => Effect.void,
        setPermissionMode: () => Effect.void,
        renameSession: (title) =>
          Effect.tryPromise({
            try: () => live.submit(`/rename ${title}`),
            catch: runnerError("renameSession"),
          }),
        stopTask: () => Effect.void,
        backgroundTasks: () => Effect.succeed(false),
        mcpServerStatus: Effect.tryPromise({
          try: async () =>
            (await listClaudeCliMcpServers(executable, cwd, environment)).map((server) => ({
              name: server.name,
              status: server.status,
            })) as never,
          catch: runnerError("mcpServerStatus"),
        }),
        contextUsageSummary: unsupported("contextUsageSummary"),
        readMcpResource: () => unsupported("readMcpResource"),
        callMcpTool: () => unsupported("callMcpTool"),
        listPermissionRules: Effect.succeed([]),
        interrupt: Effect.sync(() => live.interrupt()),
        close: Effect.sync(shutdown),
      };
      return sessionApi;
    });

    return ClaudeAgentSdkQueryRunner.of({
      allocateSessionId: random.uuidV4(),
      open,
      forkSession: () => unsupported("forkSession"),
      subagentLaunchToolUseId: () => Effect.succeed(null),
      subagentModel: () => Effect.succeed(null),
      assertComplete: Effect.void,
    });
  }),
);

export const layerTuiQueryRunner = layerTuiQueryRunnerWithPty.pipe(
  Layer.provide(NodePtyAdapter.layer),
);
