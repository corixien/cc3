// CC3 wrapper: picks the Claude transport per session. A session whose instance sets
// `transport: "tui"` (carried as CC3_CLAUDE_TRANSPORT in its environment) runs the real CLI under a
// PTY; every other session, and every lookup that only reads transcript files, uses the SDK runner.
import * as Context from "effect/Context";
import * as Effect from "effect/Effect";
import * as Layer from "effect/Layer";

import {
  ClaudeAgentSdkQueryRunner,
  layerQueryRunner as layerSdkQueryRunner,
} from "../orchestration-v2/Adapters/ClaudeAdapterV2.ts";
import { layerTuiQueryRunner } from "./ClaudeTuiQueryRunner.ts";

const wantsTui = (environment: NodeJS.ProcessEnv | undefined) =>
  (environment?.CC3_CLAUDE_TRANSPORT ?? process.env.CC3_CLAUDE_TRANSPORT) === "tui";

export const layerDispatchingQueryRunner: Layer.Layer<
  ClaudeAgentSdkQueryRunner,
  never,
  Layer.Services<typeof layerSdkQueryRunner> | Layer.Services<typeof layerTuiQueryRunner>
> = Layer.effect(
  ClaudeAgentSdkQueryRunner,
  Effect.gen(function* () {
    const sdk = Context.get(yield* Layer.build(layerSdkQueryRunner), ClaudeAgentSdkQueryRunner);
    const tui = Context.get(yield* Layer.build(layerTuiQueryRunner), ClaudeAgentSdkQueryRunner);
    return ClaudeAgentSdkQueryRunner.of({
      allocateSessionId: sdk.allocateSessionId,
      open: Effect.fn("ClaudeAgentSdkQueryRunner.open")(function* (input) {
        return yield* (wantsTui(input.options.env) ? tui : sdk).open(input);
      }),
      forkSession: Effect.fn("ClaudeAgentSdkQueryRunner.forkSession")(function* (input) {
        return yield* (wantsTui(input.environment) ? tui : sdk).forkSession(input);
      }),
      subagentLaunchToolUseId: sdk.subagentLaunchToolUseId,
      subagentModel: sdk.subagentModel,
      assertComplete: sdk.assertComplete,
    });
  }),
);
