// CC3 wrapper: byte sequences a TUI understands as keys.

export const KEY_ENTER = "\r";
export const KEY_ESCAPE = "\x1b";
export const KEY_CTRL_C = "\x03";
export const KEY_DOWN = "\x1b[B";
export const KEY_UP = "\x1b[A";
export const KEY_SHIFT_TAB = "\x1b[Z";

const PASTE_START = "\x1b[200~";
const PASTE_END = "\x1b[201~";

/** Bracketed paste keeps newlines in the text from submitting the prompt early. */
export function bracketedPaste(text: string): string {
  return `${PASTE_START}${text.replaceAll(PASTE_START, "").replaceAll(PASTE_END, "")}${PASTE_END}`;
}
