import { execFileSync } from "node:child_process";
import * as NodeFs from "node:fs";
import * as NodeOs from "node:os";
import * as NodePath from "node:path";
import { describe, expect, it } from "vitest";

import { CC3_CLAUDE_TUI_SCRIPT } from "./Manager.ts";

function runScript(env: Record<string, string>) {
  const dir = NodeFs.mkdtempSync(NodePath.join(NodeOs.tmpdir(), "cc3-tui-"));
  const bin = NodePath.join(dir, "fake-claude");
  NodeFs.writeFileSync(bin, '#!/bin/sh\nfor a in "$@"; do printf "%s\\n" "$a"; done\n', {
    mode: 0o755,
  });
  const out = execFileSync("/bin/sh", ["-c", CC3_CLAUDE_TUI_SCRIPT], {
    env: {
      PATH: "/usr/bin:/bin",
      HOME: dir,
      XDG_RUNTIME_DIR: dir,
      CLAUDE_CONFIG_DIR: dir,
      CC3_TUI_BIN: bin,
      CC3_TUI_SESSION: "abc",
      ...env,
    },
    encoding: "utf8",
  });
  return { dir, args: out.split("\n") };
}

describe("CC3 Claude TUI launch script", () => {
  it("passes flags as separate arguments and writes the helper commands", () => {
    const { dir, args } = runScript({
      CC3_TUI_SKIP_PERMISSIONS: "1",
      CC3_TUI_MCP_CONFIG: '{"mcpServers":{"cc3":{"headers":{"Authorization":"Bearer x y"}}}}',
    });
    expect(args).toContain("--dangerously-skip-permissions");
    expect(args).toContain('{"mcpServers":{"cc3":{"headers":{"Authorization":"Bearer x y"}}}}');
    expect(args).toContain("--append-system-prompt");
    expect(args.join("\n")).toContain("cc3-terminal");
    expect(args).toContain("--session-id");
    for (const name of ["xdg-open", "cc3-terminal", "cc3-terminal-output"]) {
      expect(NodeFs.statSync(NodePath.join(dir, "cc3-bin", name)).mode & 0o111).not.toBe(0);
    }
  });
});
