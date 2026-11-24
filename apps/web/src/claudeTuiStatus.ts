// CC3: what the Claude TUI of a chat is doing, read from the prefix Claude puts on the terminal title.
import type { EnvironmentId, ThreadId } from "@t3tools/contracts";

import { useClaudeTuiTitle } from "./state/terminalSessions";

export type ClaudeTuiStatus = "working" | "input" | "done";

// Claude shows a spinner glyph while it works and a star when it is idle; any other symbol asks for the user.
export function claudeTuiStatusFromTitle(title: string | null): ClaudeTuiStatus | null {
  const first = title === null ? undefined : Array.from(title)[0];
  if (first === undefined || /[\p{L}\p{N}]/u.test(first)) return null;
  if (/[◐◑◒◓⠀-⣿]/u.test(first)) return "working";
  if (first === "✳" || first === "*") return "done";
  return "input";
}

export function useClaudeTuiStatus(
  environmentId: EnvironmentId | null,
  threadId: ThreadId | null,
): ClaudeTuiStatus | null {
  return claudeTuiStatusFromTitle(useClaudeTuiTitle(environmentId, threadId));
}
