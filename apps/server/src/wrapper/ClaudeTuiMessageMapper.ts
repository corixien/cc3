// CC3 wrapper: turns Claude Code transcript events into the SDK messages `ClaudeAdapterV2` reads,
// so the adapter, orchestration and GUI stay unchanged when the CLI runs as a TUI.
import type { SDKMessage } from "@anthropic-ai/claude-agent-sdk";

import type { ClaudeTranscriptEvent, ClaudeTranscriptUsage } from "./ClaudeTranscript.ts";

export interface ClaudeTuiInit {
  readonly sessionId: string;
  readonly cwd: string;
  readonly model: string;
  readonly permissionMode: string;
  readonly version: string;
}

export interface PendingToolUse {
  readonly id: string;
  readonly name: string;
  readonly input: unknown;
}

const asSdk = (message: unknown): SDKMessage => message as SDKMessage;

export function claudeTuiInitMessage(input: ClaudeTuiInit, uuid: string): SDKMessage {
  return asSdk({
    type: "system",
    subtype: "init",
    agents: [],
    apiKeySource: "none",
    claude_code_version: input.version,
    cwd: input.cwd,
    tools: [],
    mcp_servers: [],
    model: input.model,
    permissionMode: input.permissionMode,
    slash_commands: [],
    output_style: "default",
    skills: [],
    plugins: [],
    uuid,
    session_id: input.sessionId,
  });
}

export class ClaudeTuiMessageMapper {
  private readonly sessionId: string;
  private readonly newUuid: () => string;
  private readonly openToolUses = new Map<string, PendingToolUse>();
  private lastAssistantText = "";
  private lastUsage: ClaudeTranscriptUsage | undefined;
  private lastModel: string | undefined;
  private promptUuids: string[] = [];

  constructor(sessionId: string, newUuid: () => string) {
    this.sessionId = sessionId;
    this.newUuid = newUuid;
  }

  /** The prompt the next turn answers; its uuid is echoed on the turn's result. */
  expectPrompt(uuid: string): void {
    this.promptUuids.push(uuid);
  }

  /** Tool calls the model made that have no result yet, newest last. */
  get pendingToolUses(): ReadonlyArray<PendingToolUse> {
    return [...this.openToolUses.values()];
  }

  map(event: ClaudeTranscriptEvent): ReadonlyArray<SDKMessage> {
    switch (event.kind) {
      case "assistant": {
        this.lastModel = event.model ?? this.lastModel;
        this.lastUsage = event.usage ?? this.lastUsage;
        for (const block of event.content) {
          if (block.type === "tool_use") this.openToolUses.set(block.id, block);
          if (block.type === "text") this.lastAssistantText = block.text;
        }
        return [
          asSdk({
            type: "assistant",
            message: event.rawMessage,
            parent_tool_use_id: null,
            uuid: event.uuid || this.newUuid(),
            session_id: this.sessionId,
          }),
        ];
      }
      case "tool_result": {
        this.openToolUses.delete(event.toolUseId);
        return [
          asSdk({
            type: "user",
            message: {
              role: "user",
              content: [
                {
                  type: "tool_result",
                  tool_use_id: event.toolUseId,
                  content: event.text,
                  ...(event.isError ? { is_error: true } : {}),
                },
              ],
            },
            parent_tool_use_id: null,
            uuid: this.newUuid(),
            session_id: this.sessionId,
          }),
        ];
      }
      case "turn_end": {
        const usage = this.lastUsage;
        const echoed = this.promptUuids;
        this.promptUuids = [];
        this.openToolUses.clear();
        const result = asSdk({
          type: "result",
          subtype: "success",
          is_error: false,
          duration_ms: event.durationMs ?? 0,
          duration_api_ms: event.durationMs ?? 0,
          num_turns: 1,
          result: this.lastAssistantText,
          stop_reason: "end_turn",
          total_cost_usd: 0,
          usage: {
            input_tokens: usage?.inputTokens ?? 0,
            output_tokens: usage?.outputTokens ?? 0,
            cache_read_input_tokens: usage?.cacheReadInputTokens ?? 0,
            cache_creation_input_tokens: usage?.cacheCreationInputTokens ?? 0,
          },
          modelUsage: {},
          permission_denials: [],
          ...(echoed.length > 0 ? { user_message_uuids: echoed } : {}),
          uuid: this.newUuid(),
          session_id: this.sessionId,
        });
        this.lastAssistantText = "";
        return [result];
      }
      default:
        return [];
    }
  }
}
