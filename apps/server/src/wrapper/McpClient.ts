// @effect-diagnostics nodeBuiltinImport:off globalTimers:off globalFetch:off - minimal one-shot MCP client for host-registered servers
// CC3 wrapper: calls a tool or reads a resource on an MCP server the host registered for the
// session (`mcpServers` in the query options), without going through the model. Streamable HTTP
// and stdio servers are supported; servers behind the CLI's own OAuth are not reachable this way.
import * as NodeChildProcess from "node:child_process";

type Json = Record<string, unknown>;

export interface McpServerConfig {
  readonly type?: string;
  readonly url?: string;
  readonly headers?: Record<string, string>;
  readonly command?: string;
  readonly args?: ReadonlyArray<string>;
  readonly env?: Record<string, string>;
}

const PROTOCOL_VERSION = "2025-06-18";
const CALL_TIMEOUT_MS = 60_000;

const isRecord = (value: unknown): value is Json =>
  typeof value === "object" && value !== null && !Array.isArray(value);

/** `mcp__server__tool` -> server and tool; tool names may themselves contain double underscores. */
export function splitMcpToolName(name: string): { server: string; tool: string } | null {
  const match = /^mcp__([^_]+(?:_[^_]+)*?)__(.+)$/.exec(name);
  return match?.[1] && match[2] ? { server: match[1], tool: match[2] } : null;
}

function responseFor(body: string, id: number): Json | undefined {
  const candidates = body.trimStart().startsWith("{")
    ? [body]
    : body
        .split("\n")
        .filter((line) => line.startsWith("data:"))
        .map((line) => line.slice(5).trim());
  for (const candidate of candidates) {
    try {
      const parsed: unknown = JSON.parse(candidate);
      if (isRecord(parsed) && parsed.id === id) return parsed;
    } catch {
      // not a JSON frame
    }
  }
  return undefined;
}

async function callOverHttp(config: McpServerConfig, method: string, params: Json) {
  const url = config.url as string;
  let sessionHeader: Record<string, string> = {};
  const post = async (body: Json, id: number | null) => {
    const response = await fetch(url, {
      method: "POST",
      headers: {
        "content-type": "application/json",
        accept: "application/json, text/event-stream",
        "mcp-protocol-version": PROTOCOL_VERSION,
        ...config.headers,
        ...sessionHeader,
      },
      body: JSON.stringify({ jsonrpc: "2.0", ...body }),
      signal: AbortSignal.timeout(CALL_TIMEOUT_MS),
    });
    const sessionId = response.headers.get("mcp-session-id");
    if (sessionId) sessionHeader = { "mcp-session-id": sessionId };
    const text = await response.text();
    if (!response.ok)
      throw new Error(`MCP server answered ${response.status}: ${text.slice(0, 200)}`);
    if (id === null) return undefined;
    const frame = responseFor(text, id);
    if (!frame) throw new Error("MCP server sent no response.");
    if (isRecord(frame.error)) throw new Error(String(frame.error.message ?? "MCP error"));
    return frame.result;
  };
  await post(
    {
      id: 1,
      method: "initialize",
      params: {
        protocolVersion: PROTOCOL_VERSION,
        capabilities: {},
        clientInfo: { name: "cc3", version: "1" },
      },
    },
    1,
  );
  await post({ method: "notifications/initialized" }, null);
  return post({ id: 2, method, params }, 2);
}

function callOverStdio(config: McpServerConfig, method: string, params: Json): Promise<unknown> {
  return new Promise((resolve, reject) => {
    const child = NodeChildProcess.spawn(config.command as string, [...(config.args ?? [])], {
      env: { ...process.env, ...config.env },
      stdio: ["pipe", "pipe", "ignore"],
    });
    const timer = setTimeout(() => finish(new Error("MCP call timed out.")), CALL_TIMEOUT_MS);
    let buffer = "";
    let settled = false;
    const send = (frame: Json) =>
      child.stdin.write(`${JSON.stringify({ jsonrpc: "2.0", ...frame })}\n`);
    const finish = (outcome: Error | { result: unknown }) => {
      if (settled) return;
      settled = true;
      clearTimeout(timer);
      child.kill();
      if (outcome instanceof Error) reject(outcome);
      else resolve(outcome.result);
    };
    child.on("error", finish);
    child.on("exit", () => finish(new Error("MCP server exited.")));
    child.stdout.on("data", (chunk: Buffer) => {
      buffer += chunk.toString("utf8");
      for (let end = buffer.indexOf("\n"); end >= 0; end = buffer.indexOf("\n")) {
        const line = buffer.slice(0, end);
        buffer = buffer.slice(end + 1);
        let frame: unknown;
        try {
          frame = JSON.parse(line);
        } catch {
          continue;
        }
        if (!isRecord(frame)) continue;
        if (frame.id === 1) {
          send({ method: "notifications/initialized" });
          send({ id: 2, method, params });
        } else if (frame.id === 2) {
          if (isRecord(frame.error)) finish(new Error(String(frame.error.message ?? "MCP error")));
          else finish({ result: frame.result });
        }
      }
    });
    send({
      id: 1,
      method: "initialize",
      params: {
        protocolVersion: PROTOCOL_VERSION,
        capabilities: {},
        clientInfo: { name: "cc3", version: "1" },
      },
    });
  });
}

export function callMcpServer(
  config: McpServerConfig,
  method: string,
  params: Json,
): Promise<unknown> {
  if (config.url) return callOverHttp(config, method, params);
  if (config.command) return callOverStdio(config, method, params);
  return Promise.reject(new Error("MCP server has no url or command."));
}
