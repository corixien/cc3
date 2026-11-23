// @effect-diagnostics nodeBuiltinImport:off - one-shot `claude mcp|plugin ...` calls
// CC3 wrapper: `/mcp add ...`, `/plugin marketplace add ...` and friends run as the CLI's own
// non-interactive subcommands, which cover what the TUI menus do.
import * as NodeChildProcess from "node:child_process";
import * as NodeFs from "node:fs/promises";
import * as NodePath from "node:path";

/** Splits on whitespace, keeping quoted parts together. No shell is involved. */
export function splitCommandArguments(text: string): Array<string> {
  const parts: Array<string> = [];
  let current = "";
  let quote: string | null = null;
  let started = false;
  for (const char of text) {
    if (quote !== null) {
      if (char === quote) quote = null;
      else current += char;
    } else if (char === '"' || char === "'") {
      quote = char;
      started = true;
    } else if (/\s/.test(char)) {
      if (started) parts.push(current);
      current = "";
      started = false;
    } else {
      current += char;
      started = true;
    }
  }
  if (started) parts.push(current);
  return parts;
}

/** The `claude` argv for a slash command that has a CLI subcommand; null for everything else. */
export function claudeManagementArgv(command: string): Array<string> | null {
  const match = /^\/(mcp|plugins?|marketplace)(?:\s+(.*))?$/is.exec(command.trim());
  if (!match?.[1]) return null;
  const rest = splitCommandArguments(match[2] ?? "");
  if (rest.length === 0) return null;
  const name = match[1].toLowerCase();
  if (name === "mcp") return ["mcp", ...rest];
  if (name === "marketplace") return ["plugin", "marketplace", ...rest];
  return ["plugin", ...rest];
}

export function runClaudeManagementCommand(
  executablePath: string,
  argv: ReadonlyArray<string>,
  cwd: string,
  environment: NodeJS.ProcessEnv,
  timeoutMs = 180_000,
): Promise<string> {
  return new Promise((resolve, reject) => {
    NodeChildProcess.execFile(
      executablePath,
      [...argv],
      { cwd, env: environment, timeout: timeoutMs, maxBuffer: 4 * 1024 * 1024 },
      (error, stdout, stderr) => {
        const text = [stdout, stderr]
          .map((part) => part.trim())
          .filter(Boolean)
          .join("\n");
        if (error && text.length === 0) reject(error);
        else resolve(text.length > 0 ? text : "Done.");
      },
    );
  });
}

export const PLUGIN_CATALOG_COMMAND = "/plugin catalog";

/** JSON list of every plugin offered by the configured marketplaces, for the plugin browser. */
export async function readClaudePluginCatalog(
  executablePath: string,
  cwd: string,
  environment: NodeJS.ProcessEnv,
): Promise<string> {
  const listed = await runClaudeManagementCommand(
    executablePath,
    ["plugin", "marketplace", "list", "--json"],
    cwd,
    environment,
    30_000,
  );
  const marketplaces: ReadonlyArray<{ name?: string; installLocation?: string }> =
    JSON.parse(listed);
  const catalog: Array<{
    id: string;
    name: string;
    marketplace: string;
    description: string;
    category: string;
  }> = [];
  for (const marketplace of marketplaces) {
    if (!marketplace.name || !marketplace.installLocation) continue;
    const manifest = await NodeFs.readFile(
      NodePath.join(marketplace.installLocation, ".claude-plugin", "marketplace.json"),
      "utf8",
    ).catch(() => null);
    if (manifest === null) continue;
    const plugins: ReadonlyArray<{ name?: string; description?: string; category?: string }> =
      JSON.parse(manifest).plugins ?? [];
    for (const plugin of plugins) {
      if (!plugin.name) continue;
      catalog.push({
        id: `${plugin.name}@${marketplace.name}`,
        name: plugin.name,
        marketplace: marketplace.name,
        description: (plugin.description ?? "").slice(0, 240),
        category: plugin.category ?? "",
      });
    }
  }
  return JSON.stringify(catalog);
}
