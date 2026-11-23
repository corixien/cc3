import { describe, expect, it } from "vitest";

import { claudeManagementArgv, splitCommandArguments } from "./claudeManagementCommand.ts";

describe("claudeManagementArgv", () => {
  it.each([
    [
      "/mcp add --transport http sentry https://mcp.sentry.dev/mcp",
      ["mcp", "add", "--transport", "http", "sentry", "https://mcp.sentry.dev/mcp"],
    ],
    ["/mcp remove sentry", ["mcp", "remove", "sentry"]],
    [
      "/plugin marketplace add anthropics/claude-code",
      ["plugin", "marketplace", "add", "anthropics/claude-code"],
    ],
    ["/plugins install foo@bar", ["plugin", "install", "foo@bar"]],
    ["/marketplace add owner/repo", ["plugin", "marketplace", "add", "owner/repo"]],
    ["/mcp", null],
    ["/plugin", null],
    ["/context", null],
  ])("%s", (command, expected) => {
    expect(claudeManagementArgv(command)).toEqual(expected);
  });

  it("keeps quoted arguments together", () => {
    expect(splitCommandArguments(`add-json x '{"a": 1}'`)).toEqual(["add-json", "x", '{"a": 1}']);
  });
});
