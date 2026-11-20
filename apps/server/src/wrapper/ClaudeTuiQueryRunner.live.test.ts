// @effect-diagnostics nodeBuiltinImport:off - live check against the installed CLI
// Live check against the installed Claude CLI; run with CC3_TUI_LIVE=1.
import { describe, expect, it } from "vite-plus/test";
import * as Effect from "effect/Effect";
import * as Stream from "effect/Stream";
import * as SecureRandom from "effect/crypto/SecureRandom";
import * as NodeServices from "@effect/platform-node/NodeServices";
import * as HostProcess from "@t3tools/shared/HostProcess";
import * as Layer from "effect/Layer";

import { ClaudeAgentSdkQueryRunner } from "../orchestration-v2/Adapters/ClaudeAdapterV2.ts";
import { layerTuiQueryRunner } from "./ClaudeTuiQueryRunner.ts";

describe.skipIf(process.env.CC3_TUI_LIVE !== "1")("ClaudeTuiQueryRunner (live)", () => {
  it("runs a prompt through the real TUI and ends the turn with a result", async () => {
    const asked: string[] = [];
    const program = Effect.gen(function* () {
      const runner = yield* ClaudeAgentSdkQueryRunner;
      const sessionId = yield* runner.allocateSessionId;
      const session = yield* runner.open({
        threadId: "t" as never,
        providerSessionId: "p" as never,
        options: {
          model: "haiku",
          tools: { type: "preset", preset: "claude_code" },
          permissionMode: "default",
          sessionId,
          cwd: process.cwd().replace(/\/apps\/server$/, ""),
          canUseTool: async (name: string) => {
            asked.push(name);
            return { behavior: "allow" };
          },
        } as never,
      });
      yield* session.offer({
        type: "user",
        message: {
          role: "user",
          content:
            "Use the Write tool to create /tmp/cc3-tui-perm.txt containing hi, then reply with only DONE.",
        },
        parent_tool_use_id: null,
        uuid: "00000000-0000-4000-8000-000000000001",
      } as never);
      const seen: string[] = [];
      yield* session.messages.pipe(
        Stream.tap((message) =>
          Effect.sync(
            () =>
              void seen.push(`${message.type}:${(message as { subtype?: string }).subtype ?? ""}`),
          ),
        ),
        Stream.takeUntil((message) => message.type === "result"),
        Stream.runDrain,
      );
      yield* session.close;
      return seen;
    });
    const layer = layerTuiQueryRunner.pipe(
      Layer.provide(
        Layer.mergeAll(
          NodeServices.layer,
          SecureRandom.layerWebCrypto,
          Layer.succeed(HostProcess.Platform, "linux"),
          Layer.succeed(HostProcess.Architecture, "x64"),
        ),
      ),
    );
    const seen = await Effect.runPromise(program.pipe(Effect.provide(layer)));
    console.log(seen, asked);
    expect(asked).toContain("Write");
    expect(seen[0]).toBe("system:init");
    expect(seen.at(-1)).toBe("result:success");
    expect(seen).toContain("assistant:");
  }, 240_000);
});
