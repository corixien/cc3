// @effect-diagnostics nodeBuiltinImport:off globalTimers:off - tests use real files and timers
import * as NodeFs from "node:fs";
import * as NodeOs from "node:os";
import * as NodePath from "node:path";

import { describe, expect, it } from "vite-plus/test";
import type { PtyProcess } from "@t3tools/shared/PtyAdapter";

import { claudeTranscriptPath, parseClaudeTranscriptLine } from "./ClaudeTranscript.ts";
import { bracketedPaste, KEY_ENTER } from "./ptyKeys.ts";
import { isClaudeInputReady, isClaudeTrustPrompt, stripAnsi } from "./ScreenBuffer.ts";
import { TranscriptTail } from "./TranscriptTail.ts";
import { scrubWrapperEnvironment } from "./wrapperEnvironment.ts";
import { WrapperSession } from "./WrapperSession.ts";

describe("scrubWrapperEnvironment", () => {
  it("drops inherited Claude markers and keeps user settings", () => {
    const env = scrubWrapperEnvironment({
      CLAUDE_CODE_CHILD_SESSION: "1",
      CLAUDECODE: "1",
      CLAUDE_AGENT_SDK_VERSION: "x",
      CLAUDE_CONFIG_DIR: "/c",
      PATH: "/bin",
    });
    expect(env).toEqual({ CLAUDE_CONFIG_DIR: "/c", PATH: "/bin", TERM: "xterm-256color" });
  });
});

describe("terminal text", () => {
  it("wraps pasted text and neutralises embedded paste markers", () => {
    expect(bracketedPaste("a\nb")).toBe("\x1b[200~a\nb\x1b[201~");
    expect(bracketedPaste("x\x1b[201~y")).toBe("\x1b[200~xy\x1b[201~");
  });

  it("strips escape sequences and detects the ready prompt and trust dialog", () => {
    expect(stripAnsi("\x1b[31mred\x1b[0m\x1b]0;title\x07!")).toBe("red!");
    expect(isClaudeInputReady('────\n❯ Try "write a test"\n────')).toBe(true);
    expect(isClaudeInputReady("❯No,exit\nYes,Itrustthisfolder\nEntertoconfirm·Esctocancel")).toBe(
      false,
    );
    expect(isClaudeTrustPrompt("Quicksafetycheck:Isthisaproject")).toBe(true);
    expect(isClaudeTrustPrompt("welcome")).toBe(false);
  });
});

describe("parseClaudeTranscriptLine", () => {
  it("reads a prompt, an assistant message and the turn end", () => {
    const user = parseClaudeTranscriptLine(
      JSON.stringify({ type: "user", uuid: "u1", message: { role: "user", content: "hi" } }),
    );
    expect(user).toEqual([{ kind: "user_prompt", uuid: "u1", text: "hi", timestamp: undefined }]);

    const assistant = parseClaudeTranscriptLine(
      JSON.stringify({
        type: "assistant",
        uuid: "a1",
        message: {
          id: "msg_1",
          model: "claude-haiku-5-5",
          stop_reason: "end_turn",
          content: [
            { type: "thinking", thinking: "" },
            { type: "text", text: "pong" },
            { type: "tool_use", id: "t1", name: "Bash", input: { command: "ls" } },
          ],
          usage: { input_tokens: 2, output_tokens: 31, cache_read_input_tokens: 5 },
        },
      }),
    );
    expect(assistant).toMatchObject([
      {
        kind: "assistant",
        messageId: "msg_1",
        stopReason: "end_turn",
        usage: { inputTokens: 2, outputTokens: 31, cacheReadInputTokens: 5 },
      },
    ]);
    expect(assistant[0]?.kind === "assistant" && assistant[0].content.map((b) => b.type)).toEqual([
      "thinking",
      "text",
      "tool_use",
    ]);

    expect(
      parseClaudeTranscriptLine(
        JSON.stringify({ type: "system", subtype: "turn_duration", durationMs: 3116 }),
      ),
    ).toEqual([{ kind: "turn_end", durationMs: 3116 }]);
  });

  it("reads tool results and skips sidechains, meta lines, garbage and unknown types", () => {
    expect(
      parseClaudeTranscriptLine(
        JSON.stringify({
          type: "user",
          message: {
            role: "user",
            content: [
              { type: "tool_result", tool_use_id: "t1", content: [{ type: "text", text: "ok" }] },
            ],
          },
        }),
      ),
    ).toEqual([{ kind: "tool_result", toolUseId: "t1", text: "ok", isError: false }]);
    expect(
      parseClaudeTranscriptLine(
        JSON.stringify({ type: "user", isSidechain: true, message: { content: "x" } }),
      ),
    ).toEqual([]);
    expect(parseClaudeTranscriptLine("not json")).toEqual([]);
    expect(parseClaudeTranscriptLine(JSON.stringify({ type: "cost-state" }))).toEqual([]);
  });

  it("derives the transcript path from the working directory", () => {
    expect(
      claudeTranscriptPath({ configDir: "/h/.claude", cwd: "/home/m/t3-code/cc3", sessionId: "s" }),
    ).toBe("/h/.claude/projects/-home-m-t3-code-cc3/s.jsonl");
  });
});

describe("TranscriptTail", () => {
  it("returns only complete lines and survives truncation", () => {
    const dir = NodeFs.mkdtempSync(NodePath.join(NodeOs.tmpdir(), "tail-"));
    const file = NodePath.join(dir, "t.jsonl");
    const tail = new TranscriptTail({ path: file });
    expect(tail.poll()).toEqual([]);
    NodeFs.writeFileSync(file, '{"a":1}\n{"b":');
    expect(tail.poll()).toEqual(['{"a":1}']);
    NodeFs.appendFileSync(file, "2}\n");
    expect(tail.poll()).toEqual(['{"b":2}']);
    NodeFs.writeFileSync(file, '{"c":3}\n');
    expect(tail.poll()).toEqual(['{"c":3}']);
    const fromEnd = new TranscriptTail({ path: file, fromEnd: true });
    expect(fromEnd.poll()).toEqual([]);
    NodeFs.rmSync(dir, { recursive: true });
  });
});

function fakePty() {
  const written: string[] = [];
  let data: ((chunk: string) => void) | undefined;
  let kills = 0;
  const process: PtyProcess = {
    pid: 1,
    write: (chunk) => void written.push(String(chunk)),
    resize: () => undefined,
    kill: () => void kills++,
    onData: (callback) => {
      data = callback;
      return () => undefined;
    },
    onExit: () => () => undefined,
  };
  return { process, written, emit: (chunk: string) => data?.(chunk), kills: () => kills };
}

describe("WrapperSession", () => {
  it("answers the trust dialog, waits for the prompt and submits pasted text", async () => {
    const pty = fakePty();
    const dir = NodeFs.mkdtempSync(NodePath.join(NodeOs.tmpdir(), "ws-"));
    const session = new WrapperSession({
      process: pty.process,
      transcript: new TranscriptTail({ path: NodePath.join(dir, "x.jsonl") }),
      isInputReady: isClaudeInputReady,
      answerStartupDialog: (screen) => (isClaudeTrustPrompt(screen) ? "\x1b[B\r" : undefined),
    });
    pty.emit("Quicksafetycheck: trust\nEntertoconfirm");
    const ready = session.waitUntilReady(3_000);
    setTimeout(() => pty.emit("\x1b[2J❯ Try something\n"), 700);
    await ready;
    expect(pty.written[0]).toBe("\x1b[B\r");
    await session.submit("hello");
    expect(pty.written.slice(-2)).toEqual([bracketedPaste("hello"), KEY_ENTER]);
    session.interrupt();
    expect(pty.written.at(-1)).toBe("\x1b");
    session.close();
    expect(pty.kills()).toBe(1);
    NodeFs.rmSync(dir, { recursive: true });
  });
});
