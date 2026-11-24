// CC3: the folder each Claude chat last moved to (`/cd`, or the file panel's path field), so a file panel opened later starts there.
const byThread = new Map<string, string>();

export const normalizeCwd = (path: string) => path.replace(/(.)\/+$/, "$1");
export const getClaudeCwd = (threadKey: string) => byThread.get(threadKey);
export const setClaudeCwd = (threadKey: string, path: string) => {
  byThread.set(threadKey, normalizeCwd(path));
};

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
