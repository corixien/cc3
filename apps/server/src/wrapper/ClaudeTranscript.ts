// CC3 wrapper: turns lines of a Claude Code transcript (`~/.claude/projects/<cwd>/<id>.jsonl`)
// into the few events the wrapper needs. Unknown line types are ignored so a newer CLI that adds
// bookkeeping lines does not break a session.

export interface ClaudeTranscriptUsage {
  readonly inputTokens: number;
  readonly outputTokens: number;
  readonly cacheReadInputTokens: number;
  readonly cacheCreationInputTokens: number;
}

export type ClaudeTranscriptContentBlock =
  | { readonly type: "text"; readonly text: string }
  | { readonly type: "thinking"; readonly thinking: string }
  | {
      readonly type: "tool_use";
      readonly id: string;
      readonly name: string;
      readonly input: unknown;
    };

export type ClaudeTranscriptEvent =
  | {
      readonly kind: "user_prompt";
      readonly uuid: string;
      readonly text: string;
      readonly timestamp: string | undefined;
    }
  | {
      readonly kind: "assistant";
      readonly uuid: string;
      readonly messageId: string;
      readonly model: string | undefined;
      readonly content: ReadonlyArray<ClaudeTranscriptContentBlock>;
      /** The API message as the CLI wrote it, so the adapter sees every field unchanged. */
      readonly rawMessage: Record<string, unknown>;
      readonly stopReason: string | null;
      readonly usage: ClaudeTranscriptUsage | undefined;
      readonly timestamp: string | undefined;
    }
  | {
      readonly kind: "tool_result";
      readonly toolUseId: string;
      readonly text: string;
      readonly isError: boolean;
    }
  | { readonly kind: "turn_end"; readonly durationMs: number | undefined }
  | { readonly kind: "permission_mode"; readonly mode: string }
  | { readonly kind: "title"; readonly title: string };

type Json = Record<string, unknown>;

const isRecord = (value: unknown): value is Json =>
  typeof value === "object" && value !== null && !Array.isArray(value);
const asString = (value: unknown): string | undefined =>
  typeof value === "string" ? value : undefined;
const asNumber = (value: unknown): number => (typeof value === "number" ? value : 0);

function textOf(content: unknown): string {
  if (typeof content === "string") return content;
  if (!Array.isArray(content)) return "";
  return content
    .map((block) => (isRecord(block) && block.type === "text" ? (asString(block.text) ?? "") : ""))
    .join("");
}

function parseUsage(value: unknown): ClaudeTranscriptUsage | undefined {
  if (!isRecord(value)) return undefined;
  return {
    inputTokens: asNumber(value.input_tokens),
    outputTokens: asNumber(value.output_tokens),
    cacheReadInputTokens: asNumber(value.cache_read_input_tokens),
    cacheCreationInputTokens: asNumber(value.cache_creation_input_tokens),
  };
}

function parseBlocks(content: unknown): ReadonlyArray<ClaudeTranscriptContentBlock> {
  if (!Array.isArray(content)) return [];
  const blocks: ClaudeTranscriptContentBlock[] = [];
  for (const block of content) {
    if (!isRecord(block)) continue;
    if (block.type === "text") blocks.push({ type: "text", text: asString(block.text) ?? "" });
    else if (block.type === "thinking")
      blocks.push({ type: "thinking", thinking: asString(block.thinking) ?? "" });
    else if (block.type === "tool_use")
      blocks.push({
        type: "tool_use",
        id: asString(block.id) ?? "",
        name: asString(block.name) ?? "",
        input: block.input,
      });
  }
  return blocks;
}

/** Returns the events one transcript line carries (a user line with tool results carries several). */
export function parseClaudeTranscriptLine(line: string): ReadonlyArray<ClaudeTranscriptEvent> {
  let entry: unknown;
  try {
    entry = JSON.parse(line);
  } catch {
    return [];
  }
  if (!isRecord(entry)) return [];
  const timestamp = asString(entry.timestamp);
  const uuid = asString(entry.uuid) ?? "";

  switch (entry.type) {
    case "user": {
      if (entry.isSidechain === true || entry.isMeta === true) return [];
      const message = isRecord(entry.message) ? entry.message : undefined;
      if (!message) return [];
      const content = message.content;
      if (Array.isArray(content)) {
        const results = content.flatMap((block): ClaudeTranscriptEvent[] =>
          isRecord(block) && block.type === "tool_result"
            ? [
                {
                  kind: "tool_result",
                  toolUseId: asString(block.tool_use_id) ?? "",
                  text: textOf(block.content),
                  isError: block.is_error === true,
                },
              ]
            : [],
        );
        if (results.length > 0) return results;
      }
      const text = textOf(content);
      return text.length > 0 ? [{ kind: "user_prompt", uuid, text, timestamp }] : [];
    }
    case "assistant": {
      if (entry.isSidechain === true) return [];
      const message = isRecord(entry.message) ? entry.message : undefined;
      if (!message) return [];
      return [
        {
          kind: "assistant",
          uuid,
          messageId: asString(message.id) ?? uuid,
          model: asString(message.model),
          content: parseBlocks(message.content),
          rawMessage: message,
          stopReason: asString(message.stop_reason) ?? null,
          usage: parseUsage(message.usage),
          timestamp,
        },
      ];
    }
    case "system":
      return entry.subtype === "turn_duration"
        ? [
            {
              kind: "turn_end",
              durationMs: typeof entry.durationMs === "number" ? entry.durationMs : undefined,
            },
          ]
        : [];
    case "permission-mode": {
      const mode = asString(entry.permissionMode);
      return mode ? [{ kind: "permission_mode", mode }] : [];
    }
    case "ai-title": {
      const title = asString(entry.aiTitle);
      return title ? [{ kind: "title", title }] : [];
    }
    default:
      return [];
  }
}

/** Where Claude Code stores a session transcript for a working directory. */
export function claudeTranscriptPath(input: {
  readonly configDir: string;
  readonly cwd: string;
  readonly sessionId: string;
}): string {
  const project = input.cwd.replaceAll(/[^a-zA-Z0-9]/g, "-");
  return `${input.configDir}/projects/${project}/${input.sessionId}.jsonl`;
}
