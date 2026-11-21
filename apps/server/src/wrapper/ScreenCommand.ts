// @effect-diagnostics globalDate:off globalTimers:off - polling a PTY screen until it settles
// CC3 wrapper: runs a slash command that the CLI handles locally and reads what it printed.
// Such commands never reach the transcript, so the text comes from the headless screen.
import { KEY_ESCAPE } from "./ptyKeys.ts";
import type { WrapperSession } from "./WrapperSession.ts";

const sleep = (ms: number) => new Promise<void>((resolve) => setTimeout(resolve, ms));

/** Output the command printed, read once the screen has stopped changing. */
export async function captureCommandOutput(
  session: WrapperSession,
  command: string,
): Promise<string> {
  let previous = "";
  let stableSince = Date.now();
  // Some commands (/context) run a short model call; wait for the spinner to go.
  const deadline = Date.now() + 60_000;
  while (Date.now() < deadline) {
    await sleep(300);
    const current = session.screen.lines.join("\n");
    if (current !== previous) {
      previous = current;
      stableSince = Date.now();
    } else if (Date.now() - stableSince >= 1_500 && !/esc to interrupt/i.test(current)) break;
  }
  const lines = session.screen.lines;
  // The echoed prompt line, not a later line that merely mentions the command.
  const echo = lines.findLastIndex((line) => line.trim() === `❯ ${command}`);
  // A dialog replaces the transcript view, so its body starts below the last full-width rule.
  const rule = lines.findLastIndex((line) => /[▔─━]{40,}/.test(line));
  const start = echo >= 0 ? echo + 1 : rule + 1;
  const body: string[] = [];
  for (const line of lines.slice(start)) {
    if (/^[\s]*[─━]{8,}/.test(line)) break;
    if (/^\s*[↑↓]\s*$/.test(line)) continue;
    body.push(line.replace(/^\s*⎿\s?/, "").replace(/\s+$/, ""));
  }
  // Close a dialog the command left open, so the next command types into the prompt.
  const dialogOpen = () =>
    /esc to (cancel|exit|close)|enter to (select|confirm)/i.test(
      session.screen.lines.join("\n").slice(-800),
    );
  for (let attempt = 0; attempt < 10 && dialogOpen(); attempt++) {
    session.write(KEY_ESCAPE);
    await sleep(300);
  }
  const text = body.join("\n").trim();
  return text.length > 0 ? text : "(The command printed nothing.)";
}

/** Types the command into the TUI and returns its output. */
export async function runCommandOnScreen(
  session: WrapperSession,
  command: string,
): Promise<string> {
  await session.submit(command);
  return captureCommandOutput(session, command);
}
