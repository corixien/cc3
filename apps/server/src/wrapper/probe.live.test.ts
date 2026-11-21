// @effect-diagnostics nodeBuiltinImport:off globalDate:off - live check
import { describe, expect, it } from "vite-plus/test";
import * as Effect from "effect/Effect";
import * as Schema from "effect/Schema";
import * as NodeServices from "@effect/platform-node/NodeServices";
import { ClaudeSettings } from "@t3tools/contracts";

import { probeClaudeWorkspaceSnapshot } from "../provider/ClaudeProvider.ts";

describe.skipIf(process.env.CC3_TUI_LIVE !== "1")("workspace probe (live)", () => {
  it("lists the connectors the CLI shows", async () => {
    const settings = Schema.decodeSync(ClaudeSettings)({});
    const started = Date.now();
    const snapshot = await Effect.runPromise(
      probeClaudeWorkspaceSnapshot(
        settings,
        { instanceId: "claudeAgent" } as never,
        process.cwd(),
      ).pipe(Effect.provide(NodeServices.layer)) as never,
    );
    const mcp = (snapshot as { mcpServers?: unknown }).mcpServers;
    require("node:fs").writeFileSync(
      "/tmp/claude-1000/probe.json",
      JSON.stringify(
        {
          ms: Date.now() - started,
          mcp,
          pending: (snapshot as never as { slashCommandsPending: boolean }).slashCommandsPending,
        },
        null,
        1,
      ),
    );
    expect(mcp).toBeTruthy();
  }, 120_000);
});
