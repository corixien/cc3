import { describe, expect, it } from "vitest";

import { isLiveCallerShell } from "./threadAccess.ts";

const idleShell = {
  archivedAt: null,
  activeRunId: null,
  providerInstanceId: "claudeAgent",
  pendingBackgroundTasks: [],
} as never;

describe("isLiveCallerShell", () => {
  it("needs a running turn, except for a terminal credential", () => {
    expect(isLiveCallerShell(idleShell, "claudeAgent" as never)).toBe(false);
    expect(isLiveCallerShell(idleShell, "claudeAgent" as never, true)).toBe(true);
  });

  it("never accepts an archived thread", () => {
    const archived = { ...(idleShell as object), archivedAt: "2026-01-01" } as never;
    expect(isLiveCallerShell(archived, "claudeAgent" as never, true)).toBe(false);
  });
});
