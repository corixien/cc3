// @effect-diagnostics nodeBuiltinImport:off - synchronous tail over a plain file; wrapped in Effect by the profile layer
// CC3 wrapper: reads complete JSON lines appended to a CLI transcript file.
import * as NodeFs from "node:fs";

export interface TranscriptTailOptions {
  readonly path: string;
  /** Start at the end of the file instead of replaying what is already there. */
  readonly fromEnd?: boolean;
}

/**
 * Pull-based tail: each `poll` returns the complete lines added since the last call. A trailing
 * partial line is held back until its newline arrives, and a truncated file restarts at zero.
 */
export class TranscriptTail {
  private offset = 0;
  private partial = "";
  private readonly options: TranscriptTailOptions;

  constructor(options: TranscriptTailOptions) {
    this.options = options;
    if (options.fromEnd) {
      try {
        this.offset = NodeFs.statSync(options.path).size;
      } catch {
        this.offset = 0;
      }
    }
  }

  poll(): ReadonlyArray<string> {
    let size: number;
    try {
      size = NodeFs.statSync(this.options.path).size;
    } catch {
      return [];
    }
    if (size < this.offset) {
      this.offset = 0;
      this.partial = "";
    }
    if (size === this.offset) return [];
    const fd = NodeFs.openSync(this.options.path, "r");
    try {
      const buffer = Buffer.alloc(size - this.offset);
      const read = NodeFs.readSync(fd, buffer, 0, buffer.length, this.offset);
      this.offset += read;
      const text = this.partial + buffer.subarray(0, read).toString("utf8");
      const lines = text.split("\n");
      this.partial = lines.pop() ?? "";
      return lines.filter((line) => line.length > 0);
    } finally {
      NodeFs.closeSync(fd);
    }
  }
}
