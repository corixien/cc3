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

export interface WorkCtx {
  readonly repo: string;
  /** The checkout the Changes and Branches tabs work on, or `null` when the repository is not on this computer. */
  readonly dir: string | null;
  readonly revision: number;
  readonly chatFolder: string | null;
  readonly defaultBranch: string | null;
  readonly onChanged: () => void;
  readonly toast: (type: "error" | "success", title: string, description?: string) => void;
  /** Pastes a request into Claude's input without sending it. */
  readonly askClaude: (text: string) => void;
  /** Moves the Claude session into a folder (`/cd`). */
  readonly connectClaude: (folder: string) => void;
}

export interface WtStatus {
  branch: string;
  upstream: string | null;
  ahead: number;
  behind: number;
  files: Array<{ path: string; x: string; y: string }>;
}

export const relativeTime = (iso: string) => {
  const seconds = Math.round((Date.parse(iso) - Date.now()) / 1000);
  if (Number.isNaN(seconds)) return "";
  const formatter = new Intl.RelativeTimeFormat(undefined, { numeric: "auto" });
  const units: Array<[Intl.RelativeTimeFormatUnit, number]> = [
    ["year", 31_536_000],
    ["month", 2_592_000],
    ["day", 86_400],
    ["hour", 3600],
    ["minute", 60],
  ];
  for (const [unit, size] of units) {
    if (Math.abs(seconds) >= size) return formatter.format(Math.round(seconds / size), unit);
  }
  return formatter.format(seconds, "second");
};

/** `~` for the home folder, otherwise the path as it is. */
export const tidyPath = (path: string) => path.replace(/^\/home\/[^/]+/, "~");
