// CC3: gives the Claude TUI of a chat the same T3 MCP server (browser, preview, threads) that SDK sessions get.
import { ProviderInstanceId, ThreadId } from "@t3tools/contracts";
import * as Effect from "effect/Effect";
import * as Schema from "effect/Schema";

import type * as McpSessionRegistry from "../mcp/McpSessionRegistry.ts";

export const CC3_TUI_MCP_ENV = "CC3_TUI_MCP_CONFIG";

const encodeJson = Schema.encodeEffect(Schema.fromJsonString(Schema.Unknown));

export const makeTuiMcpLauncher = (
  registry: McpSessionRegistry.McpSessionRegistryShape | undefined,
) => {
  // One credential per chat: a new one on every attach would change the env and restart the TUI.
  const configByThread = new Map<string, string>();
  return Effect.fn("cc3TuiMcp.withMcpEnv")(function* <
    Input extends {
      readonly threadId: string;
      readonly env?: Record<string, string> | undefined;
      readonly providerInstanceId?: string | undefined;
    },
  >(input: Input) {
    if (registry === undefined || !input.env?.CC3_TUI_SESSION) return input;
    let config = configByThread.get(input.threadId);
    if (config === undefined) {
      const { config: session } = yield* registry.issue({
        threadId: ThreadId.make(input.threadId),
        providerInstanceId: ProviderInstanceId.make(input.providerInstanceId ?? "claudeAgent"),
        terminal: true,
      });
      config = yield* encodeJson({
        mcpServers: {
          cc3: {
            type: "http",
            url: session.endpoint,
            headers: { Authorization: session.authorizationHeader },
          },
        },
      }).pipe(Effect.orDie);
      configByThread.set(input.threadId, config);
    } else {
      yield* registry.touch(ThreadId.make(input.threadId));
    }
    return { ...input, env: { ...input.env, [CC3_TUI_MCP_ENV]: config } };
  });
};
