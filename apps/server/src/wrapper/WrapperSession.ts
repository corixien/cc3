// @effect-diagnostics globalDate:off globalTimers:off - promise-based PTY session; wrapped in Effect by the profile layer
// CC3 wrapper: one CLI running as its interactive TUI under a PTY. The session owns the process,
// the screen copy, input and the transcript tail; provider profiles decide how to start the CLI
// and how to read its transcript.
import type { PtyProcess } from "@t3tools/shared/PtyAdapter";

import { bracketedPaste, KEY_CTRL_C, KEY_ENTER, KEY_ESCAPE } from "./ptyKeys.ts";
import { ScreenBuffer } from "./ScreenBuffer.ts";
import { TranscriptTail } from "./TranscriptTail.ts";

export interface WrapperSessionOptions {
  readonly process: PtyProcess;
  readonly transcript: TranscriptTail;
  /** True when the screen shows the CLI waiting for a prompt. */
  readonly isInputReady: (screen: string) => boolean;
  /** Answers start-up dialogs (trust prompt, update notice); returns the keys to send, if any. */
  readonly answerStartupDialog?: (screen: string) => string | undefined;
}

export class WrapperSession {
  readonly screen = new ScreenBuffer();
  private readonly screenListeners = new Set<(chunk: string) => void>();
  private readonly stopData: () => void;
  private exited = false;
  private readonly options: WrapperSessionOptions;

  constructor(options: WrapperSessionOptions) {
    this.options = options;
    this.stopData = options.process.onData((chunk) => {
      this.screen.push(chunk);
      for (const listener of this.screenListeners) listener(chunk);
    });
    options.process.onExit(() => {
      this.exited = true;
    });
  }

  get hasExited(): boolean {
    return this.exited;
  }

  /** Raw terminal output, for showing the live TUI in a popup. */
  onScreenData(listener: (chunk: string) => void): () => void {
    this.screenListeners.add(listener);
    return () => this.screenListeners.delete(listener);
  }

  /** Resolves once the CLI accepts a prompt, answering start-up dialogs on the way. */
  async waitUntilReady(timeoutMs: number): Promise<void> {
    const deadline = Date.now() + timeoutMs;
    while (Date.now() < deadline) {
      if (this.exited) throw new Error("The CLI exited before it was ready.");
      const screen = this.screen.tail;
      const answer = this.options.answerStartupDialog?.(screen);
      if (answer !== undefined) {
        // The dialog text stays in the buffer, so clear it after answering to see what follows.
        this.screen.clear();
        this.options.process.write(answer);
        await delay(400);
      } else if (this.options.isInputReady(screen)) {
        return;
      }
      await delay(150);
    }
    throw new Error(
      `The CLI did not become ready in time. Screen:\n${this.screen.lines.slice(-25).join("\n")}`,
    );
  }

  /** Types a prompt (or slash command) into the TUI and submits it. */
  async submit(text: string): Promise<void> {
    this.options.process.write(bracketedPaste(text));
    await delay(250);
    this.options.process.write(KEY_ENTER);
  }

  write(data: string): void {
    this.options.process.write(data);
  }

  interrupt(): void {
    this.options.process.write(KEY_ESCAPE);
  }

  /** Two Ctrl-C quit the TUI; used when the escape key did not stop a running tool. */
  forceInterrupt(): void {
    this.options.process.write(KEY_CTRL_C);
  }

  resize(cols: number, rows: number): void {
    this.options.process.resize(cols, rows);
  }

  /** Complete transcript lines written since the last call. */
  pollTranscript(): ReadonlyArray<string> {
    return this.options.transcript.poll();
  }

  close(): void {
    this.stopData();
    this.screenListeners.clear();
    if (!this.exited) this.options.process.kill();
  }
}

const delay = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));
