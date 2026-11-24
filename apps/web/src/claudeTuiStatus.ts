// CC3: what the Claude TUI of a chat is doing, read from the prefix Claude puts on the terminal title.
import { useSyncExternalStore } from "react";

export type ClaudeTuiStatus = "working" | "input" | "done";

const statuses = new Map<string, ClaudeTuiStatus>();
const listeners = new Set<() => void>();

// Claude shows a spinner glyph while it works and a star when it is idle; any other symbol asks for the user.
export function claudeTuiStatusFromTitle(title: string): ClaudeTuiStatus | null {
  const first = Array.from(title)[0];
  if (first === undefined || /[\p{L}\p{N}]/u.test(first)) return null;
  if (/[◐◑◒◓⠀-⣿]/u.test(first)) return "working";
  if (first === "✳" || first === "*") return "done";
  return "input";
}

export function setClaudeTuiStatus(threadId: string, status: ClaudeTuiStatus): void {
  if (statuses.get(threadId) === status) return;
  statuses.set(threadId, status);
  for (const listener of listeners) listener();
}

export function useClaudeTuiStatus(threadId: string): ClaudeTuiStatus | null {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => statuses.get(threadId) ?? null,
    () => null,
  );
}
