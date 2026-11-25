// CC3: GitHub panel of the right sidebar. Browse any repository and branch through the user's own `gh`,
// read its commits, switch the chat's checkout, link folders, pull files, and hand files to Claude.
import { scopedThreadKey } from "@t3tools/client-runtime/environment";
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
  LockIcon,
  MessageSquarePlusIcon,
  MoreHorizontalIcon,
  PlusIcon,
  RefreshCwIcon,
  SearchIcon,
} from "lucide-react";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Input } from "~/components/ui/input";
import { Menu, MenuItem, MenuPopup, MenuSeparator, MenuTrigger } from "~/components/ui/menu";
import { Spinner } from "~/components/ui/spinner";
import { toastManager } from "~/components/ui/toast";
import { setClaudeCwd, useClaudeCwd } from "~/claudeCwd";
import { useGitActivity } from "~/githubActivity";
import { writeTextToClipboard } from "~/hooks/useCopyToClipboard";
import { readLocalApi } from "~/localApi";
import { newProjectId } from "~/lib/utils";
import { sourceControlEnvironment } from "~/state/sourceControl";
import { terminalEnvironment } from "~/state/terminal";
import { useAtomCommand } from "~/state/use-atom-command";
import { CLAUDE_TUI_TERMINAL_ID } from "~/types";

import { Cc3GitHubPull } from "./Cc3GitHubPull";
import { callGh } from "./ghBridge";
import {
  type FileData,
  type LocalInfo,
  type RepoSummary,
  type Tree,
  shortName,
  useGh,
} from "./ghHooks";
import { GitHubCommits } from "./GitHubCommits";
import { CloneForm, DeleteForm, LinkFolderForm, NewBranchForm, NewRepoForm } from "./GitHubForms";
import { SimpleSelect } from "./GitHubUi";
import { useRepoFolderLinks } from "./repoFolderLinks";

type Dialog = "repo" | "branch" | "link" | "clone" | "delete" | "pull" | null;

export default function Cc3GitHubPanel(props: {
  readonly threadRef: ScopedThreadRef;
  readonly cwd: string | null;
}) {
  const gt = useGT();
  const threadKey = scopedThreadKey(props.threadRef);
  const chatFolder = useClaudeCwd(threadKey) ?? props.cwd;
  const folderLinks = useRepoFolderLinks();
  const [manualRevision, setRevision] = useState(0);
  const revision = manualRevision + useGitActivity();
  const local = useGh<LocalInfo>(chatFolder ? { op: "local", cwd: chatFolder } : null, revision);
  const account = useGh<{ login: string }>({ op: "status" });
  const repos = useGh<RepoSummary[]>({ op: "repos" }, revision);

  const [listMode, setListMode] = useState(false);
  const [query, setQuery] = useState("");
  const [typedRepo, setTypedRepo] = useState<string | null>(null);
  const repo = listMode ? null : (typedRepo ?? local.data?.repo ?? null);
  const repoInfo = useGh<RepoSummary>(repo ? { op: "repo", repo } : null);
  const branches = useGh<string[]>(repo ? { op: "branches", repo } : null, revision);
  const [pickedBranch, setPickedBranch] = useState<string | null>(null);
  const isLocalRepo = repo !== null && repo === local.data?.repo;
  const defaultBranch = repoInfo.data?.defaultBranchRef?.name ?? null;
  const branch =
    pickedBranch && branches.data?.includes(pickedBranch)
      ? pickedBranch
      : ((isLocalRepo ? local.data?.branch : null) ?? defaultBranch ?? branches.data?.[0] ?? null);

  const [tab, setTab] = useState<"files" | "commits">("files");
  const [dialog, setDialog] = useState<Dialog>(null);
  const [busy, setBusy] = useState(false);
  const [path, setPath] = useState("");
  const [openFilePath, setOpenFilePath] = useState<string | null>(null);
  const tree = useGh<Tree>(
    repo && branch && tab === "files" ? { op: "tree", repo, ref: branch, path } : null,
  );
  const file = useGh<FileData>(
    repo && branch && openFilePath ? { op: "file", repo, ref: branch, path: openFilePath } : null,
  );

  const startProjectClone = useAtomCommand(sourceControlEnvironment.startProjectClone, {
    reportFailure: false,
  });
  const writeTerminal = useAtomCommand(terminalEnvironment.write, { reportFailure: false });

  const login = account.data?.login ?? "";
  const displayName = (full: string) => (full.startsWith(`${login}/`) ? shortName(full) : full);
  const home = props.cwd ? /^\/home\/[^/]+/.exec(props.cwd)?.[0] : undefined;
  const folderIsFree = chatFolder !== null && local.data !== null && local.data.repo === null;
  const linkedFolder = repo ? (folderLinks.links[repo] ?? (isLocalRepo ? chatFolder : null)) : null;
  const chatOnLinked = linkedFolder !== null && linkedFolder === chatFolder;
  // Taking the chat out of a repository returns it to the chat's own project folder, or the home folder.
  const leaveTarget = (props.cwd !== chatFolder ? props.cwd : home) ?? null;
  const githubUrl = repo
    ? `https://github.com/${repo}${branch ? `/tree/${branch}${path ? `/${path}` : ""}` : ""}`
    : null;
  const fileUrl =
    repo && branch && openFilePath
      ? `https://github.com/${repo}/blob/${branch}/${openFilePath}`
      : null;

  const toast = (type: "error" | "success", title: string, description?: string) =>
    toastManager.add({ type, title, ...(description ? { description } : {}) });
  const refresh = () => setRevision((value) => value + 1);
  const sendToClaude = (data: string) => {
    void writeTerminal({
      environmentId: props.threadRef.environmentId,
      input: { threadId: props.threadRef.threadId, terminalId: CLAUDE_TUI_TERMINAL_ID, data },
    }).then((result) => {
      if (result._tag === "Failure" && !isAtomCommandInterrupted(result)) {
        toast(
          "error",
          gt("Unable to reach Claude"),
          gt("Open a Claude chat for this project and try again."),
        );
      }
    });
  };
  /** Moves the Claude session into a folder, as if the user had typed /cd there. */
  const connectClaude = (folder: string) => {
    setClaudeCwd(threadKey, folder);
    sendToClaude(`\u0015/cd ${folder}\r`);
  };

  const chooseRepo = (value: string) => {
    const next = value
      .trim()
      .replace(/^https:\/\/github\.com\//, "")
      .replace(/\.git$/, "");
    if (!/^[\w.-]+\/[\w.-]+$/.test(next)) return;
    setTypedRepo(next);
    setListMode(false);
    setDialog(null);
    setQuery("");
    setPickedBranch(null);
    setPath("");
    setOpenFilePath(null);
    setTab("files");
  };
  const backToList = () => {
    setListMode(true);
    setDialog(null);
    setPath("");
    setOpenFilePath(null);
  };

  const switchBranch = async () => {
    if (!chatFolder || !branch) return;
    setBusy(true);
    const needsFetch = !local.data?.branches.includes(branch);
    const fetched = needsFetch ? await callGh({ op: "fetch", cwd: chatFolder }) : null;
    const result =
      fetched && !fetched.ok ? fetched : await callGh({ op: "switch", cwd: chatFolder, branch });
    setBusy(false);
    if (result.ok) {
      toast("success", gt("Switched to {branch}", { branch }));
      refresh();
    } else {
      toast("error", gt("Could not switch branch"), result.error);
    }
  };

  const clone = async (target: string) => {
    if (!repo) return;
    setBusy(true);
    const result = await startProjectClone({
      environmentId: props.threadRef.environmentId,
      input: {
        projectId: newProjectId(),
        title: shortName(repo),
        // oxlint-disable-next-line react/purity -- runs in an event handler
        createdAt: new Date().toISOString(),
        repository: repo,
        destinationPath: target,
      },
    });
    setBusy(false);
    if (result._tag === "Failure") {
      if (!isAtomCommandInterrupted(result)) {
        toast("error", gt("Clone failed"), String(squashAtomCommandFailure(result)));
      }
      return;
    }
    folderLinks.link(repo, target);
    toast("success", gt("Cloning {repo}", { repo }), target);
    setDialog(null);
  };

  const createRepository = async (input: {
    name: string;
    description: string;
    visibility: "private" | "public";
    inFolder: boolean;
  }) => {
    const useFolder = input.inFolder && folderIsFree && chatFolder;
    setBusy(true);
    const result = await callGh({
      op: "createRepo",
      name: input.name,
      visibility: input.visibility,
      description: input.description,
      ...(useFolder ? { cwd: chatFolder } : { parent: home ?? chatFolder }),
    });
    setBusy(false);
    if (!result.ok) {
      toast("error", gt("Could not create the repository"), result.error);
      return;
    }
    const created = result.data as { repo: string | null; path: string };
    toast("success", gt("Created {repo}", { repo: created.repo ?? input.name }), created.path);
    if (created.repo) folderLinks.link(created.repo, created.path);
    setDialog(null);
    setTypedRepo(useFolder ? null : created.repo);
    setListMode(false);
    setPickedBranch(null);
    setPath("");
    setOpenFilePath(null);
    refresh();
    if (!useFolder) connectClaude(created.path);
  };

  const createBranch = async (input: { name: string; publish: boolean }) => {
    if (!repo || !branch) return;
    setBusy(true);
    const result = await callGh(
      isLocalRepo && chatFolder
        ? {
            op: "createBranch",
            cwd: chatFolder,
            name: input.name,
            base: branch,
            push: input.publish,
          }
        : { op: "createRemoteBranch", repo, name: input.name, base: branch },
    );
    setBusy(false);
    if (!result.ok) {
      toast("error", gt("Could not create the branch"), result.error);
      return;
    }
    toast(
      "success",
      gt("Created {branch}", { branch: input.name }),
      isLocalRepo ? gt("Claude is now on this branch.") : gt("Clone the repository to work on it."),
    );
    setDialog(null);
    setPickedBranch(input.name);
    refresh();
  };

  const saveLink = async (folderInput: string, useInChat: boolean) => {
    if (!repo) return;
    const folder = folderInput.replace(/(.)\/+$/, "$1");
    const result = await callGh({ op: "local", cwd: folder });
    if (!result.ok) {
      toast("error", gt("Could not link the folder"), result.error);
      return;
    }
    const info = result.data as LocalInfo;
    if (!info.exists) {
      toast("error", gt("Could not link the folder"), gt("This folder does not exist."));
      return;
    }
    if (info.repo && info.repo !== repo) {
      toast(
        "error",
        gt("Could not link the folder"),
        gt("This folder belongs to {repo}.", { repo: info.repo }),
      );
      return;
    }
    folderLinks.link(repo, folder);
    setDialog(null);
    toast("success", gt("Linked {folder}", { folder }));
    if (useInChat) connectClaude(folder);
  };

  const deleteRepository = async () => {
    if (!repo) return;
    setBusy(true);
    const result = await callGh({ op: "deleteRepo", repo });
    setBusy(false);
    if (!result.ok) {
      toast("error", gt("Could not delete the repository"), result.error);
      return;
    }
    toast("success", gt("Deleted {repo}", { repo }));
    folderLinks.unlink(repo);
    setTypedRepo(null);
    refresh();
    backToList();
  };

  // ---------------------------------------------------------------- list of repositories
  if (!repo) {
    const needle = query.trim().toLowerCase();
    const matches = (repos.data ?? []).filter(
      (entry) =>
        !needle ||
        shortName(entry.nameWithOwner).toLowerCase().includes(needle) ||
        (entry.description ?? "").toLowerCase().includes(needle),
    );
    const typedFull = /^(?:https:\/\/github\.com\/)?[\w.-]+\/[\w.-]+(?:\.git)?$/.test(query.trim());
    return (
      <div className="flex min-h-0 flex-1 flex-col text-xs" data-cc3-github-panel>
        {dialog === "repo" ? (
          <div className="overflow-auto">
            <NewRepoForm
              folderFree={folderIsFree}
              parent={home ?? chatFolder ?? "~"}
              busy={busy}
              onCancel={() => setDialog(null)}
              onSubmit={(input) => void createRepository(input)}
            />
          </div>
        ) : (
          <>
            <div className="flex shrink-0 flex-col gap-2 border-b border-border/60 p-3">
              <div className="flex items-center gap-2">
                <div className="min-w-0 flex-1">
                  <div className="text-sm font-medium text-foreground">{gt("Repositories")}</div>
                  <div className="truncate text-[11px] text-muted-foreground">
                    {login
                      ? gt("Signed in as {login}", { login })
                      : account.loading
                        ? ""
                        : gt("Not signed in. Run `gh auth login` in a terminal.")}
                  </div>
                </div>
                <Button size="sm" variant="outline" onClick={() => setDialog("repo")}>
                  <PlusIcon className="size-3.5" />
                  {gt("New")}
                </Button>
                <Button size="icon-xs" variant="ghost" aria-label={gt("Refresh")} onClick={refresh}>
                  <RefreshCwIcon className="size-3.5" />
                </Button>
              </div>
              <div className="relative">
                <SearchIcon className="pointer-events-none absolute top-1/2 left-2.5 z-10 size-3.5 -translate-y-1/2 text-muted-foreground" />
                <Input
                  size="compact"
                  className="[&_input]:pl-8"
                  value={query}
                  spellCheck={false}
                  placeholder={gt("Search repositories")}
                  aria-label={gt("Search repositories")}
                  onChange={(event) => setQuery(event.target.value)}
                  onKeyDown={(event) => {
                    if (event.key === "Enter" && typedFull) chooseRepo(query);
                  }}
                />
              </div>
            </div>
            <div className="min-h-0 flex-1 overflow-auto">
              {repos.loading && !repos.data ? (
                <div className="p-3">
                  <Spinner className="size-4" />
                </div>
              ) : null}
              {repos.error ? <div className="p-3 text-destructive">{repos.error}</div> : null}
              <ul className="flex flex-col py-1">
                {matches.map((entry) => (
                  <li key={entry.nameWithOwner}>
                    <button
                      type="button"
                      className="flex w-full flex-col items-start gap-0.5 px-3 py-2 text-left hover:bg-accent/60"
                      onClick={() => chooseRepo(entry.nameWithOwner)}
                    >
                      <span className="flex w-full items-center gap-1.5 text-xs font-medium text-foreground">
                        <span className="truncate">{displayName(entry.nameWithOwner)}</span>
                        {entry.isPrivate ? (
                          <LockIcon className="size-3 shrink-0 text-muted-foreground" />
                        ) : null}
                      </span>
                      {entry.description ? (
                        <span className="w-full truncate text-[11px] text-muted-foreground">
                          {entry.description}
                        </span>
                      ) : null}
                    </button>
                  </li>
                ))}
              </ul>
              {typedFull && !matches.length ? (
                <button
                  type="button"
                  className="mx-3 my-2 rounded-md border border-border/70 px-3 py-2 text-left text-xs hover:bg-accent/60"
                  onClick={() => chooseRepo(query)}
                >
                  {gt("Open {repo}", { repo: query.trim() })}
                </button>
              ) : null}
              {!matches.length && !repos.loading && !typedFull && repos.data ? (
                <div className="p-3 text-muted-foreground">
                  {gt("No match. Type owner/name and press Enter to open any repository.")}
                </div>
              ) : null}
            </div>
          </>
        )}
      </div>
    );
  }

  // ---------------------------------------------------------------- one repository
  const crumbs = path ? path.split("/") : [];
  const branchItems = (branches.data ?? (branch ? [branch] : [])).map((name) => ({
    value: name,
    label: `${name}${name === defaultBranch ? ` (${gt("default")})` : ""}${
      isLocalRepo && name === local.data?.branch ? ` · ${gt("checked out")}` : ""
    }`,
  }));
  const dialogView =
    dialog === "branch" && branch ? (
      <NewBranchForm
        base={branch}
        local={isLocalRepo}
        busy={busy}
        onCancel={() => setDialog(null)}
        onSubmit={(input) => void createBranch(input)}
      />
    ) : dialog === "link" ? (
      <LinkFolderForm
        initial={linkedFolder ?? `${home ?? ""}/${shortName(repo)}`}
        chatFolder={chatFolder}
        onCancel={() => setDialog(null)}
        onSubmit={(folder, useInChat) => void saveLink(folder, useInChat)}
      />
    ) : dialog === "clone" ? (
      <CloneForm
        repo={repo}
        initial={`${home ?? ""}/${shortName(repo)}`}
        onCancel={() => setDialog(null)}
        onSubmit={(target) => void clone(target)}
      />
    ) : dialog === "delete" ? (
      <DeleteForm
        repo={repo}
        busy={busy}
        onCancel={() => setDialog(null)}
        onSubmit={() => void deleteRepository()}
      />
    ) : null;

  return (
    <div className="flex min-h-0 flex-1 flex-col text-xs" data-cc3-github-panel>
      <div className="flex shrink-0 flex-col gap-2 border-b border-border/60 p-3">
        <div className="flex items-center gap-1.5">
          <Button
            size="icon-xs"
            variant="ghost"
            aria-label={gt("All repositories")}
            onClick={backToList}
          >
            <ArrowLeftIcon className="size-3.5" />
          </Button>
          <span
            className="min-w-0 flex-1 truncate text-sm font-medium text-foreground"
            title={repo}
          >
            {displayName(repo)}
          </span>
          {repoInfo.data?.isPrivate ? (
            <LockIcon className="size-3 shrink-0 text-muted-foreground" />
          ) : null}
          <Menu>
            <MenuTrigger
              render={
                <Button size="icon-xs" variant="ghost" aria-label={gt("Repository actions")} />
              }
            >
              <MoreHorizontalIcon className="size-4" />
            </MenuTrigger>
            <MenuPopup align="end">
              <MenuItem disabled={!branch} onClick={() => setDialog("pull")}>
                {gt("Pull into folder…")}
              </MenuItem>
              <MenuItem onClick={() => setDialog("branch")}>{gt("New branch…")}</MenuItem>
              <MenuItem onClick={() => setDialog("link")}>
                {linkedFolder ? gt("Change linked folder…") : gt("Link a folder…")}
              </MenuItem>
              {!isLocalRepo ? (
                <MenuItem onClick={() => setDialog("clone")}>{gt("Clone…")}</MenuItem>
              ) : null}
              <MenuSeparator />
              <MenuItem
                onClick={() => githubUrl && void readLocalApi()?.shell.openExternal(githubUrl)}
              >
                {gt("Open on GitHub")}
              </MenuItem>
              <MenuItem onClick={() => void writeTextToClipboard(`https://github.com/${repo}.git`)}>
                {gt("Copy clone URL")}
              </MenuItem>
              {folderLinks.links[repo] ? (
                <MenuItem onClick={() => folderLinks.unlink(repo)}>{gt("Unlink folder")}</MenuItem>
              ) : null}
              <MenuSeparator />
              <MenuItem variant="destructive" onClick={() => setDialog("delete")}>
                {gt("Delete repository…")}
              </MenuItem>
            </MenuPopup>
          </Menu>
        </div>

        <div className="flex items-center gap-2">
          <SimpleSelect
            ariaLabel={gt("Branch")}
            value={branch ?? ""}
            onChange={(value) => {
              setPickedBranch(value);
              setPath("");
              setOpenFilePath(null);
            }}
            items={branchItems}
            icon={<GitBranchIcon className="size-3.5 shrink-0 text-muted-foreground" />}
            className="min-w-0 flex-1"
          />
          {isLocalRepo && branch && branch !== local.data?.branch ? (
            <Button size="sm" disabled={busy} onClick={() => void switchBranch()}>
              {busy ? <Spinner className="size-3.5" /> : gt("Switch")}
            </Button>
          ) : null}
        </div>

        <div className="flex items-center gap-2 text-[11px] text-muted-foreground">
          <FolderIcon className="size-3.5 shrink-0" />
          <span className="min-w-0 flex-1 truncate" title={linkedFolder ?? ""}>
            {linkedFolder ?? gt("No folder linked")}
          </span>
          {linkedFolder && !chatOnLinked ? (
            <Button size="xs" variant="outline" onClick={() => connectClaude(linkedFolder)}>
              {gt("Use in this chat")}
            </Button>
          ) : linkedFolder ? (
            <>
              <span>{gt("This chat is here")}</span>
              {leaveTarget ? (
                <Button size="xs" variant="outline" onClick={() => connectClaude(leaveTarget)}>
                  {gt("Leave")}
                </Button>
              ) : null}
            </>
          ) : (
            <Button size="xs" variant="outline" onClick={() => setDialog("link")}>
              {gt("Link folder")}
            </Button>
          )}
        </div>

        <div className="flex items-center gap-1 rounded-md bg-muted/50 p-0.5">
          {(["files", "commits"] as const).map((id) => (
            <button
              key={id}
              type="button"
              className={`flex-1 rounded-[5px] px-2 py-1 text-xs font-medium transition-colors ${
                tab === id
                  ? "bg-background text-foreground shadow-xs"
                  : "text-muted-foreground hover:text-foreground"
              }`}
              onClick={() => {
                setTab(id);
                setOpenFilePath(null);
                setDialog(null);
              }}
            >
              {id === "files" ? gt("Files") : gt("Commits")}
            </button>
          ))}
        </div>
      </div>

      <div className="min-h-0 flex-1 overflow-auto">
        {dialog === "pull" && branch ? (
          <Cc3GitHubPull
            repo={repo}
            branch={branch}
            source={path}
            defaultTarget={linkedFolder ?? chatFolder ?? home ?? ""}
            currentChatFolder={chatFolder}
            linkedFolder={linkedFolder}
            onLink={(folder) => folderLinks.link(repo, folder)}
            onClose={() => setDialog(null)}
            onPulled={(folder, moveClaude) => {
              setDialog(null);
              refresh();
              if (moveClaude) connectClaude(folder);
            }}
          />
        ) : dialogView ? (
          dialogView
        ) : tab === "commits" && branch ? (
          <GitHubCommits key={`${repo}@${branch}`} repo={repo} branch={branch} refresh={revision} />
        ) : openFilePath ? (
          <div className="flex min-h-0 flex-col">
            <div className="flex items-center gap-1 border-b border-border/60 px-2 py-1.5">
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
                onClick={() => fileUrl && sendToClaude(`\u001b[200~Look at ${fileUrl} \u001b[201~`)}
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
            <div className="flex flex-wrap items-center gap-1 px-3 py-1.5 text-muted-foreground">
              <button type="button" className="hover:text-foreground" onClick={() => setPath("")}>
                {shortName(repo)}
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
              <ul className="flex flex-col px-1 pb-2">
                {tree.data.entries.map((entry) => (
                  <li key={entry.path}>
                    <button
                      type="button"
                      className="flex w-full items-center gap-2 rounded-sm px-2 py-1 text-left hover:bg-accent/60"
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
