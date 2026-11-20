// @effect-diagnostics nodeBuiltinImport:off - one-shot `claude mcp list` call
// CC3 wrapper: the MCP servers `claude mcp list` reports, including claude.ai connectors and plugins.
import * as NodeChildProcess from "node:child_process";

/** Parses `claude mcp list` lines such as `name: target - ✔ Connected`. */
export function parseClaudeCliMcpList(output: string): Array<{ name: string; status: string }> {
  const servers: Array<{ name: string; status: string }> = [];
  for (const line of output.split("\n")) {
    const separator = line.indexOf(": ");
    const statusAt = line.lastIndexOf(" - ");
    if (separator <= 0 || statusAt <= separator) continue;
    const label = line.slice(statusAt + 3).toLowerCase();
    const status =
      label.includes("connected") && !label.includes("fail")
        ? "connected"
        : label.includes("auth")
          ? "needs-auth"
          : label.includes("fail")
            ? "failed"
            : "pending";
    servers.push({ name: line.slice(0, separator).trim(), status });
  }
  return servers;
}

export function listClaudeCliMcpServers(
  executablePath: string,
  cwd: string | undefined,
  environment: NodeJS.ProcessEnv,
  timeoutMs = 28_000,
): Promise<Array<{ name: string; status: string }>> {
  return new Promise((resolve) => {
    NodeChildProcess.execFile(
      executablePath,
      ["mcp", "list"],
      { ...(cwd ? { cwd } : {}), env: environment, timeout: timeoutMs },
      (error, stdout) => resolve(error ? [] : parseClaudeCliMcpList(stdout)),
    );
  });
}
