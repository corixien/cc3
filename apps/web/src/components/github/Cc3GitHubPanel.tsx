// CC3: GitHub panel of the right sidebar. Browse any repository and branch through the user's own `gh`,
// switch the chat's checkout, clone a repository as a project, and hand files to Claude.
import {
  isAtomCommandInterrupted,
  squashAtomCommandFailure,
} from "@t3tools/client-runtime/state/runtime";
import type { ScopedThreadRef } from "@t3tools/contracts";
import { useGT } from "gt-react";
import {
  ArrowLeftIcon,
  ExternalLinkIcon,
  FileIcon,
  FolderIcon,
  GitBranchIcon,
  MessageSquarePlusIcon,
  RefreshCwIcon,
} from "lucide-react";
import { useEffect, useState } from "react";

import { Button } from "~/components/ui/button";
import { Spinner } from "~/components/ui/spinner";
import { toastManager } from "~/components/ui/toast";
import { writeTextToClipboard } from "~/hooks/useCopyToClipboard";
import { readLocalApi } from "~/localApi";
import { newProjectId } from "~/lib/utils";
import { sourceControlEnvironment } from "~/state/sourceControl";
import { terminalEnvironment } from "~/state/terminal";
import { useAtomCommand } from "~/state/use-atom-command";
import { CLAUDE_TUI_TERMINAL_ID } from "~/types";

type GhResult = { ok: true; data: unknown } | { ok: false; error: string };

const callGh = (request: Record<string, unknown>): Promise<GhResult> =>
  window.desktopBridge?.github(request) ??
  Promise.resolve({ ok: false, error: "GitHub needs the CC3 desktop app." });

interface Query<T> {
  readonly data: T | null;
  readonly error: string | null;
  readonly loading: boolean;
}

/** Runs one request whenever its key changes; `null` skips it. */
function useGh<T>(request: Record<string, unknown> | null, refresh = 0): Query<T> {
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

interface RepoSummary {
  nameWithOwner: string;
  description: string | null;
  isPrivate: boolean;
  url: string;
  defaultBranchRef: { name: string } | null;
}
interface LocalInfo {
  repo: string | null;
  branch: string | null;
  branches: string[];
}
type Tree =
  | { kind: "file" }
  | {
      kind: "dir";
      entries: Array<{ name: string; path: string; type: "dir" | "file"; size: number }>;
    };
type FileData = { binary: true } | { binary: false; text: string };

const SELECT_CLASS =
  "h-7 min-w-0 flex-1 rounded-md border border-border/70 bg-background px-2 text-xs text-foreground outline-none focus:ring-1 focus:ring-ring";
const INPUT_CLASS =
  "h-7 min-w-0 flex-1 rounded-md border border-border/70 bg-background px-2 text-xs text-foreground outline-none focus:ring-1 focus:ring-ring";

export default function Cc3GitHubPanel(props: {
  readonly threadRef: ScopedThreadRef;
  readonly cwd: string | null;
}) {
  const gt = useGT();
  const [revision, setRevision] = useState(0);
  const local = useGh<LocalInfo>(props.cwd ? { op: "local", cwd: props.cwd } : null, revision);
  const account = useGh<{ login: string }>({ op: "status" });
  const repos = useGh<RepoSummary[]>({ op: "repos" });
  const [typedRepo, setTypedRepo] = useState<string | null>(null);
  const repo = typedRepo ?? local.data?.repo ?? null;
  const [repoInput, setRepoInput] = useState("");
  const repoInfo = useGh<RepoSummary>(repo ? { op: "repo", repo } : null);
  const branches = useGh<string[]>(repo ? { op: "branches", repo } : null, revision);
  const [pickedBranch, setPickedBranch] = useState<string | null>(null);
  const isLocalRepo = repo !== null && repo === local.data?.repo;
  const defaultBranch = repoInfo.data?.defaultBranchRef?.name ?? null;
  const branch =
    pickedBranch && branches.data?.includes(pickedBranch)
      ? pickedBranch
      : ((isLocalRepo ? local.data?.branch : null) ?? defaultBranch ?? branches.data?.[0] ?? null);
  const [path, setPath] = useState("");
  const [openFilePath, setOpenFilePath] = useState<string | null>(null);
  const tree = useGh<Tree>(repo && branch ? { op: "tree", repo, ref: branch, path } : null);
  const file = useGh<FileData>(
    repo && branch && openFilePath ? { op: "file", repo, ref: branch, path: openFilePath } : null,
  );
  const [switching, setSwitching] = useState(false);
  const [cloneTarget, setCloneTarget] = useState<string | null>(null);
  const startProjectClone = useAtomCommand(sourceControlEnvironment.startProjectClone, {
    reportFailure: false,
  });
  const writeTerminal = useAtomCommand(terminalEnvironment.write, { reportFailure: false });

  const home = props.cwd ? /^\/home\/[^/]+/.exec(props.cwd)?.[0] : undefined;
  const githubUrl = repo
    ? `https://github.com/${repo}${branch ? `/tree/${branch}${path ? `/${path}` : ""}` : ""}`
    : null;

  const chooseRepo = (value: string) => {
    const next = value
      .trim()
      .replace(/^https:\/\/github\.com\//, "")
      .replace(/\.git$/, "");
    if (!/^[\w.-]+\/[\w.-]+$/.test(next)) return;
    setTypedRepo(next);
    setRepoInput("");
    setPickedBranch(null);
    setPath("");
    setOpenFilePath(null);
  };
  const toast = (type: "error" | "success", title: string, description?: string) =>
    toastManager.add({ type, title, ...(description ? { description } : {}) });

  const switchBranch = async () => {
    if (!props.cwd || !branch) return;
    setSwitching(true);
    const needsFetch = !local.data?.branches.includes(branch);
    const fetched = needsFetch ? await callGh({ op: "fetch", cwd: props.cwd }) : null;
    const result =
      fetched && !fetched.ok ? fetched : await callGh({ op: "switch", cwd: props.cwd, branch });
    setSwitching(false);
    if (result.ok) {
      toast("success", gt("Switched to {branch}", { branch }));
      setRevision((value) => value + 1);
    } else {
      toast("error", gt("Could not switch branch"), result.error);
    }
  };

  const clone = async () => {
    if (!repo || !cloneTarget) return;
    const name = repo.split("/")[1] ?? repo;
    const result = await startProjectClone({
      environmentId: props.threadRef.environmentId,
      input: {
        projectId: newProjectId(),
        title: name,
        // oxlint-disable-next-line react/purity -- runs in an event handler
        createdAt: new Date().toISOString(),
        repository: repo,
        destinationPath: cloneTarget,
      },
    });
    if (result._tag === "Failure") {
      if (!isAtomCommandInterrupted(result)) {
        toast("error", gt("Clone failed"), String(squashAtomCommandFailure(result)));
      }
      return;
    }
    toast("success", gt("Cloning {repo}", { repo }), cloneTarget);
    setCloneTarget(null);
  };

  const askClaude = (text: string) => {
    void writeTerminal({
      environmentId: props.threadRef.environmentId,
      input: {
        threadId: props.threadRef.threadId,
        terminalId: CLAUDE_TUI_TERMINAL_ID,
        data: `\u001b[200~${text}\u001b[201~`,
      },
    }).then((result) => {
      if (result._tag === "Failure" && !isAtomCommandInterrupted(result)) {
        toast(
          "error",
          gt("Unable to add to chat"),
          gt("Open a Claude chat for this project and try again."),
        );
      }
    });
  };

  const crumbs = path ? path.split("/") : [];
  const fileUrl =
    githubUrl && openFilePath ? `https://github.com/${repo}/blob/${branch}/${openFilePath}` : null;

  return (
    <div className="flex min-h-0 flex-1 flex-col text-xs" data-cc3-github-panel>
      <div className="flex shrink-0 flex-col gap-2 border-b border-border/60 p-2">
        <div className="flex items-center gap-2">
          <span className="min-w-0 flex-1 truncate text-muted-foreground">
            {account.data
              ? gt("Signed in as {login}", { login: account.data.login })
              : account.loading
                ? ""
                : gt("Not signed in. Run `gh auth login` in a terminal.")}
          </span>
          <Button
            size="icon-xs"
            variant="ghost"
            aria-label={gt("Refresh")}
            onClick={() => setRevision((value) => value + 1)}
          >
            <RefreshCwIcon className="size-3.5" />
          </Button>
        </div>
        <div className="flex items-center gap-2">
          <input
            className={INPUT_CLASS}
            list="cc3-github-repos"
            value={repoInput}
            placeholder={repo ?? gt("owner/name")}
            spellCheck={false}
            aria-label={gt("Repository")}
            onChange={(event) => {
              setRepoInput(event.target.value);
              if (repos.data?.some((entry) => entry.nameWithOwner === event.target.value)) {
                chooseRepo(event.target.value);
              }
            }}
            onKeyDown={(event) => {
              if (event.key === "Enter") chooseRepo(repoInput);
            }}
          />
          <datalist id="cc3-github-repos">
            {repos.data?.map((entry) => (
              <option key={entry.nameWithOwner} value={entry.nameWithOwner}>
                {entry.description ?? ""}
              </option>
            ))}
          </datalist>
        </div>
        {repo ? (
          <div className="flex items-center gap-2">
            <GitBranchIcon className="size-3.5 shrink-0 text-muted-foreground" />
            <select
              className={SELECT_CLASS}
              value={branch ?? ""}
              aria-label={gt("Branch")}
              onChange={(event) => {
                setPickedBranch(event.target.value);
                setPath("");
                setOpenFilePath(null);
              }}
            >
              {(branches.data ?? (branch ? [branch] : [])).map((name) => (
                <option key={name} value={name}>
                  {name}
                  {name === defaultBranch ? ` (${gt("default")})` : ""}
                  {isLocalRepo && name === local.data?.branch ? ` (${gt("checked out")})` : ""}
                </option>
              ))}
            </select>
            {isLocalRepo && branch && branch !== local.data?.branch ? (
              <Button size="xs" disabled={switching} onClick={() => void switchBranch()}>
                {switching ? <Spinner className="size-3" /> : gt("Switch")}
              </Button>
            ) : null}
          </div>
        ) : null}
        {repo ? (
          <div className="flex flex-wrap items-center gap-1.5">
            {!isLocalRepo ? (
              <Button
                size="xs"
                variant="outline"
                onClick={() => setCloneTarget(`${home ?? ""}/${repo.split("/")[1] ?? "repo"}`)}
              >
                {gt("Clone")}
              </Button>
            ) : null}
            <Button
              size="xs"
              variant="ghost"
              onClick={() => githubUrl && void readLocalApi()?.shell.openExternal(githubUrl)}
            >
              <ExternalLinkIcon className="size-3" />
              {gt("Open on GitHub")}
            </Button>
            <Button
              size="xs"
              variant="ghost"
              onClick={() => void writeTextToClipboard(`https://github.com/${repo}.git`)}
            >
              {gt("Copy clone URL")}
            </Button>
          </div>
        ) : null}
        {cloneTarget !== null ? (
          <div className="flex items-center gap-2">
            <input
              className={INPUT_CLASS}
              value={cloneTarget}
              spellCheck={false}
              aria-label={gt("Clone into")}
              onChange={(event) => setCloneTarget(event.target.value)}
              onKeyDown={(event) => {
                if (event.key === "Enter") void clone();
                if (event.key === "Escape") setCloneTarget(null);
              }}
            />
            <Button size="xs" onClick={() => void clone()}>
              {gt("Clone")}
            </Button>
            <Button size="xs" variant="ghost" onClick={() => setCloneTarget(null)}>
              {gt("Cancel")}
            </Button>
          </div>
        ) : null}
      </div>

      <div className="min-h-0 flex-1 overflow-auto">
        {!repo ? (
          <div className="p-3 text-muted-foreground">
            {local.loading || repos.loading
              ? ""
              : gt("Pick one of your repositories or type owner/name to browse it.")}
            {repos.data ? (
              <ul className="mt-2 flex flex-col">
                {repos.data.slice(0, 30).map((entry) => (
                  <li key={entry.nameWithOwner}>
                    <button
                      type="button"
                      className="flex w-full flex-col items-start rounded-sm px-2 py-1 text-left hover:bg-accent"
                      onClick={() => chooseRepo(entry.nameWithOwner)}
                    >
                      <span className="text-foreground">{entry.nameWithOwner}</span>
                      {entry.description ? (
                        <span className="w-full truncate text-muted-foreground">
                          {entry.description}
                        </span>
                      ) : null}
                    </button>
                  </li>
                ))}
              </ul>
            ) : null}
            {repos.error ? <div className="mt-2 text-destructive">{repos.error}</div> : null}
          </div>
        ) : openFilePath ? (
          <div className="flex min-h-0 flex-col">
            <div className="flex items-center gap-1 border-b border-border/60 px-2 py-1">
              <Button
                size="icon-xs"
                variant="ghost"
                aria-label={gt("Back")}
                onClick={() => setOpenFilePath(null)}
              >
                <ArrowLeftIcon className="size-3.5" />
              </Button>
              <span className="min-w-0 flex-1 truncate text-foreground">{openFilePath}</span>
              <Button
                size="icon-xs"
                variant="ghost"
                aria-label={gt("Ask Claude about this file")}
                onClick={() => fileUrl && askClaude(`Look at ${fileUrl} `)}
              >
                <MessageSquarePlusIcon className="size-3.5" />
              </Button>
              <Button
                size="icon-xs"
                variant="ghost"
                aria-label={gt("Open on GitHub")}
                onClick={() => fileUrl && void readLocalApi()?.shell.openExternal(fileUrl)}
              >
                <ExternalLinkIcon className="size-3.5" />
              </Button>
            </div>
            {file.loading ? (
              <div className="p-3">
                <Spinner className="size-4" />
              </div>
            ) : file.error ? (
              <div className="p-3 text-destructive">{file.error}</div>
            ) : file.data?.binary ? (
              <div className="p-3 text-muted-foreground">{gt("Binary file.")}</div>
            ) : (
              <pre className="overflow-auto p-3 font-mono text-[11px] leading-relaxed text-foreground">
                {file.data?.text}
              </pre>
            )}
          </div>
        ) : (
          <div className="flex flex-col">
            <div className="flex flex-wrap items-center gap-1 px-2 py-1.5 text-muted-foreground">
              <button type="button" className="hover:text-foreground" onClick={() => setPath("")}>
                {repo}
              </button>
              {crumbs.map((crumb, index) => (
                <span key={crumb + index} className="flex items-center gap-1">
                  /
                  <button
                    type="button"
                    className="hover:text-foreground"
                    onClick={() => setPath(crumbs.slice(0, index + 1).join("/"))}
                  >
                    {crumb}
                  </button>
                </span>
              ))}
            </div>
            {tree.loading ? (
              <div className="p-3">
                <Spinner className="size-4" />
              </div>
            ) : tree.error ? (
              <div className="p-3 text-destructive">{tree.error}</div>
            ) : tree.data?.kind === "dir" ? (
              <ul className="flex flex-col px-1">
                {tree.data.entries.map((entry) => (
                  <li key={entry.path}>
                    <button
                      type="button"
                      className="flex w-full items-center gap-2 rounded-sm px-2 py-1 text-left hover:bg-accent"
                      onClick={() =>
                        entry.type === "dir" ? setPath(entry.path) : setOpenFilePath(entry.path)
                      }
                    >
                      {entry.type === "dir" ? (
                        <FolderIcon className="size-3.5 shrink-0 text-muted-foreground" />
                      ) : (
                        <FileIcon className="size-3.5 shrink-0 text-muted-foreground" />
                      )}
                      <span className="min-w-0 truncate text-foreground">{entry.name}</span>
                    </button>
                  </li>
                ))}
              </ul>
            ) : null}
          </div>
        )}
      </div>
    </div>
  );
}
