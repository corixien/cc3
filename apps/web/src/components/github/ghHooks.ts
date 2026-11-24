// CC3: data hook and shared types of the GitHub panel.
import { useEffect, useState } from "react";

import { callGh } from "./ghBridge";

export interface Query<T> {
  readonly data: T | null;
  readonly error: string | null;
  readonly loading: boolean;
}

/** Runs one request whenever its key changes; `null` skips it. */
export function useGh<T>(request: Record<string, unknown> | null, refresh = 0): Query<T> {
  const [state, setState] = useState<Query<T>>({ data: null, error: null, loading: false });
  const key = request === null ? null : JSON.stringify(request);
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- loads external data through the desktop host
  useEffect(() => {
    if (key === null) {
      setState({ data: null, error: null, loading: false });
      return;
    }
    let cancelled = false;
    setState((previous) => ({ ...previous, loading: true, error: null }));
    void callGh(JSON.parse(key) as Record<string, unknown>).then((result) => {
      if (cancelled) return;
      setState(
        result.ok
          ? { data: result.data as T, error: null, loading: false }
          : { data: null, error: result.error, loading: false },
      );
    });
    return () => {
      cancelled = true;
    };
  }, [key, refresh]);
  return state;
}

export interface RepoSummary {
  nameWithOwner: string;
  description: string | null;
  isPrivate: boolean;
  url: string;
  defaultBranchRef: { name: string } | null;
}
export interface LocalInfo {
  exists?: boolean;
  repo: string | null;
  branch: string | null;
  branches: string[];
}
export type Tree =
  | { kind: "file" }
  | {
      kind: "dir";
      entries: Array<{ name: string; path: string; type: "dir" | "file"; size: number }>;
    };
export type FileData = { binary: true } | { binary: false; text: string };

export const shortName = (full: string) => full.split("/")[1] ?? full;
