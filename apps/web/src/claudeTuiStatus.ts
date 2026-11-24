// CC3: what the Claude TUI of a chat is doing, read from the prefix Claude puts on the terminal title.
import type { EnvironmentId, ThreadId } from "@t3tools/contracts";

import { create } from "zustand";

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

/** Chats whose Claude finished while the user was not looking; the green dot lasts until they view the chat. */
export const useClaudeTuiUnseen = create<{
  readonly unseen: ReadonlySet<string>;
  readonly mark: (threadId: string) => void;
  readonly clear: (threadId: string) => void;
}>((set) => ({
  unseen: new Set(),
  mark: (threadId) =>
    set((state) =>
      state.unseen.has(threadId) ? state : { unseen: new Set(state.unseen).add(threadId) },
    ),
  clear: (threadId) =>
    set((state) => {
      if (!state.unseen.has(threadId)) return state;
      const next = new Set(state.unseen);
      next.delete(threadId);
      return { unseen: next };
    }),
}));
