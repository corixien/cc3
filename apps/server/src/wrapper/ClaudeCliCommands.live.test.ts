// @effect-diagnostics nodeBuiltinImport:off globalDate:off - live check against the installed CLI
import { describe, expect, it } from "vite-plus/test";
import * as Effect from "effect/Effect";
import * as Layer from "effect/Layer";
import * as NodeServices from "@effect/platform-node/NodeServices";
import * as HostProcess from "@t3tools/shared/HostProcess";
import * as PtyAdapter from "@t3tools/shared/PtyAdapter";

import * as NodePtyAdapter from "../terminal/NodePtyAdapter.ts";
import { makeClaudeCliCommandRunner } from "./ClaudeCliCommands.ts";
import { closeStandbyTui } from "./LiveTuiRegistry.ts";

describe.skipIf(process.env.CC3_TUI_LIVE !== "1")("ClaudeCliCommands (live)", () => {
  it("answers /mcp from the CLI and /cost from a standby TUI", async () => {
    const cwd = process.cwd().replace(/\/apps\/server$/, "");
    const layer = NodePtyAdapter.layer.pipe(
      Layer.provide(
        Layer.mergeAll(
          NodeServices.layer,
          Layer.succeed(HostProcess.Platform, "linux"),
          Layer.succeed(HostProcess.Architecture, "x64"),
        ),
      ),
    );
    const program = Effect.gen(function* () {
      const pty = yield* PtyAdapter.PtyAdapter;
      const run = makeClaudeCliCommandRunner({
        pty,
        binaryPath: "claude",
        environment: process.env,
      });
      const mcp = yield* run({ cwd, threadId: "thread-live", command: "/mcp" });
      const started = Date.now();
      const cost = yield* run({ cwd, threadId: "thread-live", command: "/cost" });
      const again = yield* run({ cwd, threadId: "thread-live", command: "/context" });
      return { mcp, cost, again, ms: Date.now() - started };
    });
    const result = await Effect.runPromise(program.pipe(Effect.provide(layer)) as never);
    closeStandbyTui("thread-live");
    require("node:fs").writeFileSync("/tmp/claude-1000/cli.json", JSON.stringify(result, null, 1));
    const typed = result as { mcp: { mcpServers?: unknown[] }; cost: { output: string } };
    expect(typed.mcp.mcpServers?.length).toBeGreaterThan(0);
    expect(typed.cost.output.length).toBeGreaterThan(20);
  }, 240_000);
});
