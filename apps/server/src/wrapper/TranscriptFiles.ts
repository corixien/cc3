// @effect-diagnostics nodeBuiltinImport:off - synchronous transcript file work
// CC3 wrapper: finding, forking and cross-referencing Claude Code transcript files.
import * as NodeFs from "node:fs";
import * as NodePath from "node:path";

type Json = Record<string, unknown>;
const isRecord = (value: unknown): value is Json =>
  typeof value === "object" && value !== null && !Array.isArray(value);

/** The transcript for a session id, searching every project directory of this Claude home. */
export function findClaudeTranscript(configDir: string, sessionId: string): string | null {
  const projects = NodePath.join(configDir, "projects");
  let names: string[];
  try {
    names = NodeFs.readdirSync(projects);
  } catch {
    return null;
  }
  for (const name of names) {
    const candidate = NodePath.join(projects, name, `${sessionId}.jsonl`);
    if (NodeFs.existsSync(candidate)) return candidate;
  }
  return null;
}

function promptMapPath(transcriptPath: string): string {
  return transcriptPath.replace(/\.jsonl$/, ".cc3-prompts.json");
}

/** Adapter message uuid -> uuid the CLI gave the same prompt in its transcript. */
export function readPromptMap(transcriptPath: string): Record<string, string> {
  try {
    const parsed: unknown = JSON.parse(NodeFs.readFileSync(promptMapPath(transcriptPath), "utf8"));
    return isRecord(parsed) ? (parsed as Record<string, string>) : {};
  } catch {
    return {};
  }
}

export function recordPrompt(
  transcriptPath: string,
  adapterUuid: string,
  transcriptUuid: string,
): void {
  const map = readPromptMap(transcriptPath);
  map[adapterUuid] = transcriptUuid;
  NodeFs.writeFileSync(promptMapPath(transcriptPath), JSON.stringify(map));
}

/**
 * Copies a transcript up to and including the line whose uuid matches `upTo` (or all of it) into a
 * new session, as `forkSession` does in the SDK. `upTo` may be the adapter's prompt uuid.
 */
export function forkClaudeTranscript(input: {
  readonly configDir: string;
  readonly sessionId: string;
  readonly newSessionId: string;
  readonly upTo?: string;
}): { sessionId: string; headMessageId?: string } {
  const source = findClaudeTranscript(input.configDir, input.sessionId);
  if (source === null) throw new Error(`No transcript found for session ${input.sessionId}.`);
  const promptMap = readPromptMap(source);
  const target = input.upTo === undefined ? undefined : (promptMap[input.upTo] ?? input.upTo);
  const lines = NodeFs.readFileSync(source, "utf8").split("\n").filter(Boolean);
  let end = lines.length;
  if (target !== undefined) {
    const index = lines.findIndex((line) => {
      try {
        const entry: unknown = JSON.parse(line);
        return isRecord(entry) && entry.uuid === target;
      } catch {
        return false;
      }
    });
    if (index >= 0) end = index + 1;
  }
  let head: string | undefined;
  const kept = lines.slice(0, end).map((line) => {
    try {
      const entry: unknown = JSON.parse(line);
      if (!isRecord(entry)) return line;
      if ("sessionId" in entry) entry.sessionId = input.newSessionId;
      if ("session_id" in entry) entry.session_id = input.newSessionId;
      if (typeof entry.uuid === "string" && (entry.type === "user" || entry.type === "assistant"))
        head = entry.uuid;
      return JSON.stringify(entry);
    } catch {
      return line;
    }
  });
  const destination = NodePath.join(NodePath.dirname(source), `${input.newSessionId}.jsonl`);
  NodeFs.writeFileSync(destination, `${kept.join("\n")}\n`);
  return { sessionId: input.newSessionId, ...(head === undefined ? {} : { headMessageId: head }) };
}

function subagentTranscript(configDir: string, sessionId: string, agentId: string): string | null {
  const parent = findClaudeTranscript(configDir, sessionId);
  if (parent === null) return null;
  const candidate = NodePath.join(
    parent.replace(/\.jsonl$/, ""),
    "subagents",
    `agent-${agentId}.jsonl`,
  );
  return NodeFs.existsSync(candidate) ? candidate : null;
}

/** The model a subagent's transcript says it ran on. */
export function subagentModel(
  configDir: string,
  sessionId: string,
  agentId: string,
): string | null {
  const file = subagentTranscript(configDir, sessionId, agentId);
  if (file === null) return null;
  for (const line of NodeFs.readFileSync(file, "utf8").split("\n")) {
    try {
      const entry: unknown = JSON.parse(line);
      if (isRecord(entry) && entry.type === "assistant" && isRecord(entry.message)) {
        const model = entry.message.model;
        if (typeof model === "string") return model;
      }
    } catch {
      // skip
    }
  }
  return null;
}

/** The tool call in the parent transcript that launched the subagent. */
export function subagentLaunchToolUseId(
  configDir: string,
  sessionId: string,
  agentId: string,
): string | null {
  const parent = findClaudeTranscript(configDir, sessionId);
  if (parent === null) return null;
  for (const line of NodeFs.readFileSync(parent, "utf8").split("\n")) {
    if (!line.includes(agentId)) continue;
    try {
      const entry: unknown = JSON.parse(line);
      if (!isRecord(entry) || entry.type !== "user" || !isRecord(entry.message)) continue;
      const content = entry.message.content;
      if (!Array.isArray(content)) continue;
      for (const block of content) {
        if (
          isRecord(block) &&
          block.type === "tool_result" &&
          typeof block.tool_use_id === "string"
        )
          return block.tool_use_id;
      }
    } catch {
      // skip
    }
  }
  return null;
}
