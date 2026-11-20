// CC3 wrapper: a headless terminal emulator holding what a TUI shows, for readiness checks,
// permission-mode reads and capturing the output of commands that never reach the transcript.
// The user never sees this terminal; the GUI renders the extracted text.
import xtermHeadless from "@xterm/headless";

const { Terminal } = xtermHeadless;

// oxlint-disable-next-line no-control-regex -- terminal escape sequences are the input
const ANSI = /\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07\x1b]*(?:\x07|\x1b\\)|[()][0-9A-Za-z]|[=>])/g;

export function stripAnsi(text: string): string {
  return text.replace(ANSI, "");
}

export class ScreenBuffer {
  private readonly terminal: InstanceType<typeof Terminal>;

  constructor(cols = 120, rows = 40) {
    this.terminal = new Terminal({ cols, rows, scrollback: 2_000, allowProposedApi: true });
  }

  push(chunk: string): void {
    this.terminal.write(chunk);
  }

  resize(cols: number, rows: number): void {
    this.terminal.resize(cols, rows);
  }

  /** Every line the terminal holds (scrollback and screen), trailing blanks trimmed. */
  get lines(): ReadonlyArray<string> {
    const buffer = this.terminal.buffer.active;
    const lines: string[] = [];
    for (let row = 0; row < buffer.length; row++) {
      lines.push(buffer.getLine(row)?.translateToString(true) ?? "");
    }
    while (lines.length > 0 && lines[lines.length - 1] === "") lines.pop();
    return lines;
  }

  /** The most recent output as text, for pattern checks. */
  get tail(): string {
    return this.lines.slice(-60).join("\n");
  }

  clear(): void {
    this.terminal.reset();
  }

  dispose(): void {
    this.terminal.dispose();
  }
}

/** Claude Code shows its input box (`❯`) between two rules once it accepts a prompt. */
export function isClaudeInputReady(screen: string): boolean {
  const end = screen.slice(-1_500);
  return (
    /❯/.test(screen) && !/Do you (trust|want)/i.test(end) && !/Enter\s*to\s*confirm/i.test(end)
  );
}

export function isClaudeTrustPrompt(screen: string): boolean {
  return (
    /Quick\s*safety\s*check|trust\s*this\s*folder/i.test(screen.replaceAll(/\s+/g, " ")) ||
    /Yes,\s*I\s*trust/i.test(screen)
  );
}

/** The permission mode the footer announces, e.g. `accept edits on`, or null when none shows. */
export function readClaudePermissionMode(screen: string): string | null {
  const footer = screen.split("\n").slice(-6).join("\n").toLowerCase();
  if (footer.includes("bypass permissions on")) return "bypassPermissions";
  if (footer.includes("accept edits on")) return "acceptEdits";
  if (footer.includes("plan mode on")) return "plan";
  if (footer.includes("auto mode on")) return "auto";
  if (footer.includes("manual mode on") || footer.includes("? for shortcuts")) return "default";
  return null;
}
