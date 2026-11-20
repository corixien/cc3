// CC3 wrapper: a rolling, ANSI-free copy of what a TUI printed, for readiness and prompt checks.

// oxlint-disable-next-line no-control-regex -- terminal escape sequences are the input
const ANSI = /\x1b(?:\[[0-?]*[ -/]*[@-~]|\][^\x07\x1b]*(?:\x07|\x1b\\)|[()][0-9A-Za-z]|[=>])/g;

export function stripAnsi(text: string): string {
  return text.replace(ANSI, "");
}

export class ScreenBuffer {
  private text = "";
  private readonly maxLength: number;
  constructor(maxLength = 16_000) {
    this.maxLength = maxLength;
  }

  push(chunk: string): void {
    this.text = (this.text + stripAnsi(chunk)).slice(-this.maxLength);
  }

  /** The most recent printed text, without escape sequences. */
  get tail(): string {
    return this.text;
  }

  clear(): void {
    this.text = "";
  }
}

/** Claude Code shows its input box (`❯`) between two rules once it accepts a prompt. */
export function isClaudeInputReady(screen: string): boolean {
  return (
    /❯/.test(screen) &&
    !/Do you (trust|want)/i.test(screen.slice(-1_500)) &&
    !/Entertoconfirm|Enter to confirm/.test(screen.slice(-600))
  );
}

export function isClaudeTrustPrompt(screen: string): boolean {
  return (
    /Quick\s*safety\s*check|trust\s*this\s*folder/i.test(screen.replaceAll(/\s+/g, " ")) ||
    /Yes,\s*I\s*trust|Yes,Itrust/i.test(screen)
  );
}
