// @effect-diagnostics nodeBuiltinImport:off - temp directory fixture
import * as NodeFs from "node:fs";
import * as NodeOs from "node:os";
import * as NodePath from "node:path";
import { describe, expect, it } from "vite-plus/test";

import { parseCliPassthrough, resolveConfigPath, runConfigRequest } from "./claudeConfigFiles.ts";

describe("claude config files", () => {
  it("builds scope paths and rejects names that escape the folder", () => {
    expect(resolveConfigPath({ kind: "settings", scope: "local" }, "/p", {})).toBe(
      "/p/.claude/settings.local.json",
    );
    expect(resolveConfigPath({ kind: "memory", scope: "project" }, "/p", {})).toBe("/p/CLAUDE.md");
    expect(
      resolveConfigPath({ kind: "agent", scope: "user", name: "a" }, "/p", {
        CLAUDE_CONFIG_DIR: "/c",
      }),
    ).toBe("/c/agents/a.md");
    expect(() =>
      resolveConfigPath({ kind: "agent", scope: "user", name: "../x" }, "/p", {}),
    ).toThrow();
  });

  it("writes, lists, reads and deletes, and refuses invalid settings JSON", async () => {
    const cwd = NodeFs.mkdtempSync(NodePath.join(NodeOs.tmpdir(), "cc3-config-"));
    await runConfigRequest(
      { op: "write", kind: "agent", scope: "project", name: "rev", text: "hi" },
      cwd,
      {},
    );
    expect(await runConfigRequest({ op: "list", kind: "agent", scope: "project" }, cwd, {})).toBe(
      '["rev"]',
    );
    expect(
      await runConfigRequest({ op: "read", kind: "agent", scope: "project", name: "rev" }, cwd, {}),
    ).toBe("hi");
    await runConfigRequest({ op: "delete", kind: "agent", scope: "project", name: "rev" }, cwd, {});
    expect(await runConfigRequest({ op: "list", kind: "agent", scope: "project" }, cwd, {})).toBe(
      "[]",
    );
    await expect(
      runConfigRequest({ op: "write", kind: "settings", scope: "project", text: "{" }, cwd, {}),
    ).rejects.toThrow();
  });

  it("only passes allowed subcommands", () => {
    expect(parseCliPassthrough("/cc3-cli auth status")).toEqual(["auth", "status"]);
    expect(parseCliPassthrough("/cc3-cli doctor")).toEqual(["doctor"]);
    expect(parseCliPassthrough("/cc3-cli purge")).toBeNull();
  });
});
