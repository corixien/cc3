// CC3: the folder each Claude chat last moved to (`/cd`, or the file panel's path field), so a file panel opened later starts there.
import { useSyncExternalStore } from "react";

const byThread = new Map<string, string>();
const listeners = new Set<() => void>();

export const normalizeCwd = (path: string) => path.replace(/(.)\/+$/, "$1");
export const getClaudeCwd = (threadKey: string) => byThread.get(threadKey);
export const setClaudeCwd = (threadKey: string, path: string) => {
  byThread.set(threadKey, normalizeCwd(path));
  for (const listener of listeners) listener();
};

/** The folder Claude is in for a chat, or `undefined` while it is still in the project folder. */
export function useClaudeCwd(threadKey: string): string | undefined {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => byThread.get(threadKey),
  );
}

/** Resolves a typed folder against the current one; `~` is the home folder and `..` is applied. */
export function resolveTypedPath(typed: string, current: string, home: string | undefined) {
  const joined = /^~(\/|$)/.test(typed)
    ? `${home ?? ""}${typed.slice(1)}`
    : typed.startsWith("/")
      ? typed
      : `${current}/${typed}`;
  const parts: string[] = [];
  for (const part of joined.split("/")) {
    if (part === "..") parts.pop();
    else if (part !== "" && part !== ".") parts.push(part);
  }
  return `/${parts.join("/")}`;
}
