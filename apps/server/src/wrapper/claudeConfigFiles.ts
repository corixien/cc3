// @effect-diagnostics nodeBuiltinImport:off - reads and writes Claude Code's own config files
// CC3 wrapper: settings, memory, agents, commands and output styles that Claude Code keeps in plain
// files. The GUI names a scope and a kind; the path is built here so the client never sends one.
import * as NodeFs from "node:fs/promises";
import * as NodeOs from "node:os";
import * as NodePath from "node:path";

export const CONFIG_COMMAND_PREFIX = "/cc3-config ";

export type ConfigScope = "user" | "project" | "local";
export type ConfigKind = "settings" | "memory" | "agent" | "command" | "style";

export interface ConfigRequest {
  readonly op: "read" | "write" | "list" | "delete";
  readonly kind: ConfigKind;
  readonly scope: ConfigScope;
  readonly name?: string | undefined;
  readonly text?: string | undefined;
}

const NAME = /^[A-Za-z0-9][A-Za-z0-9._-]*$/;
const FOLDER: Record<"agent" | "command" | "style", string> = {
  agent: "agents",
  command: "commands",
  style: "output-styles",
};

function configRoot(scope: ConfigScope, cwd: string, environment: NodeJS.ProcessEnv): string {
  return scope === "user"
    ? (environment.CLAUDE_CONFIG_DIR ?? NodePath.join(NodeOs.homedir(), ".claude"))
    : NodePath.join(cwd, ".claude");
}

/** Path of one config file, or of the folder holding many (agent, command, style without a name). */
export function resolveConfigPath(
  request: Pick<ConfigRequest, "kind" | "scope" | "name">,
  cwd: string,
  environment: NodeJS.ProcessEnv,
): string {
  const root = configRoot(request.scope, cwd, environment);
  switch (request.kind) {
    case "settings":
      return NodePath.join(
        root,
        request.scope === "local" ? "settings.local.json" : "settings.json",
      );
    case "memory":
      return request.scope === "user"
        ? NodePath.join(root, "CLAUDE.md")
        : NodePath.join(cwd, request.scope === "local" ? "CLAUDE.local.md" : "CLAUDE.md");
    default: {
      const folder = NodePath.join(root, FOLDER[request.kind]);
      if (request.name === undefined) return folder;
      if (!NAME.test(request.name)) throw new Error(`Invalid name: ${request.name}`);
      return NodePath.join(folder, `${request.name}.md`);
    }
  }
}

export async function runConfigRequest(
  request: ConfigRequest,
  cwd: string,
  environment: NodeJS.ProcessEnv,
): Promise<string> {
  const path = resolveConfigPath(request, cwd, environment);
  switch (request.op) {
    case "list": {
      const entries = await NodeFs.readdir(path).catch(() => [] as Array<string>);
      return JSON.stringify(
        entries.filter((entry) => entry.endsWith(".md")).map((entry) => entry.slice(0, -3)),
      );
    }
    case "read":
      return await NodeFs.readFile(path, "utf8").catch(() => "");
    case "write": {
      const text = request.text ?? "";
      if (request.kind === "settings" && text.trim() !== "") JSON.parse(text);
      await NodeFs.mkdir(NodePath.dirname(path), { recursive: true });
      await NodeFs.writeFile(path, text, "utf8");
      return "Saved.";
    }
    case "delete":
      if (request.name === undefined && request.kind !== "settings" && request.kind !== "memory") {
        throw new Error("A name is required.");
      }
      await NodeFs.rm(path, { force: true });
      return "Deleted.";
  }
}

export function parseConfigCommand(command: string): ConfigRequest | null {
  if (!command.startsWith(CONFIG_COMMAND_PREFIX)) return null;
  const value: unknown = JSON.parse(command.slice(CONFIG_COMMAND_PREFIX.length));
  return value as ConfigRequest;
}

const CLI_COMMAND = /^\/cc3-cli\s+(auth|doctor|auto-mode|update)(?:\s+(.*))?$/s;

/** `claude auth`, `doctor`, `auto-mode` and `update`, for the account and diagnostics tabs. */
export function parseCliPassthrough(command: string): Array<string> | null {
  const match = CLI_COMMAND.exec(command.trim());
  return match?.[1] ? [match[1], ...(match[2]?.trim().split(/\s+/).filter(Boolean) ?? [])] : null;
}
