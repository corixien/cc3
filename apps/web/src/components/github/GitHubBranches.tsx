// CC3: branches of the checkout Claude works in, and worktrees: one folder per branch, so Claude can work on
// several branches in parallel without them getting in each other's way.
import { useGT } from "gt-react";
import { CheckIcon, FolderPlusIcon, PlusIcon, Trash2Icon } from "lucide-react";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Checkbox } from "~/components/ui/checkbox";
import { Input } from "~/components/ui/input";
import { Spinner } from "~/components/ui/spinner";

import { callGh } from "./ghBridge";
import { type WorkCtx, shortName, tidyPath, useGh } from "./ghHooks";
import { Field, FormCard, SimpleSelect } from "./GitHubUi";

interface LocalBranch {
  current: boolean;
  name: string;
  upstream: string | null;
  track: string;
  when: string;
  subject: string;
}
interface Worktree {
  path: string;
  branch: string | null;
  main: boolean;
}

const siblingFolder = (mainDir: string, repo: string, branch: string) =>
  `${mainDir.replace(/\/[^/]+$/, "")}/${shortName(repo)}-${branch.replace(/[^\w.-]+/g, "-")}`;

function NewBranchCard(props: {
  readonly ctx: WorkCtx;
  readonly base: string;
  readonly mainDir: string;
  readonly onDone: () => void;
}) {
  const gt = useGT();
  const { ctx } = props;
  const [name, setName] = useState("");
  const [where, setWhere] = useState("folder");
  const [publish, setPublish] = useState(true);
  const [busy, setBusy] = useState(false);
  const submit = async () => {
    const branch = name.trim();
    if (!branch || !ctx.dir) return;
    setBusy(true);
    let result;
    let target = ctx.dir;
    if (where === "own") {
      target = siblingFolder(props.mainDir, ctx.repo, branch);
      result = await callGh({
        op: "worktreeAdd",
        cwd: ctx.dir,
        branch,
        base: props.base,
        create: true,
        path: target,
      });
    } else {
      result = await callGh({
        op: "createBranch",
        cwd: ctx.dir,
        name: branch,
        base: props.base,
        push: publish,
      });
    }
    setBusy(false);
    if (!result.ok) {
      ctx.toast("error", gt("Could not create the branch"), result.error);
      return;
    }
    ctx.toast("success", gt("Created {branch}", { branch }), where === "own" ? target : undefined);
    if (where === "own") ctx.connectClaude(target);
    ctx.onChanged();
    props.onDone();
  };
  return (
    <FormCard
      title={gt("New branch from {base}", { base: props.base })}
      submitLabel={gt("Create branch")}
      busy={busy}
      disabled={!name.trim()}
      onCancel={props.onDone}
      onSubmit={() => void submit()}
    >
      <Field label={gt("Branch name")}>
        <Input
          size="compact"
          autoFocus
          value={name}
          spellCheck={false}
          placeholder="feature/login"
          onChange={(event) => setName(event.target.value)}
        />
      </Field>
      <Field label={gt("Where Claude works on it")}>
        <SimpleSelect
          ariaLabel={gt("Where")}
          value={where}
          onChange={setWhere}
          items={[
            { value: "folder", label: gt("In this folder (switch)") },
            { value: "own", label: gt("In its own folder (parallel work)") },
          ]}
        />
      </Field>
      {where === "folder" ? (
        <label className="flex items-center gap-2 text-xs text-muted-foreground">
          <Checkbox checked={publish} onCheckedChange={(checked) => setPublish(checked === true)} />
          {gt("Publish to GitHub")}
        </label>
      ) : (
        <p className="text-xs text-muted-foreground">
          {gt("A second folder is created next to this one. Claude moves there.")}
        </p>
      )}
    </FormCard>
  );
}

export function GitHubBranches(props: {
  readonly ctx: WorkCtx;
  readonly currentBranch: string | null;
}) {
  const gt = useGT();
  const { ctx } = props;
  const branches = useGh<LocalBranch[]>(
    ctx.dir ? { op: "localBranches", cwd: ctx.dir } : null,
    ctx.revision,
  );
  const trees = useGh<Worktree[]>(ctx.dir ? { op: "worktrees", cwd: ctx.dir } : null, ctx.revision);
  const [creating, setCreating] = useState(false);
  const [busyKey, setBusyKey] = useState<string | null>(null);
  const [folderFor, setFolderFor] = useState<string | null>(null);
  const [folderPath, setFolderPath] = useState("");

  if (!ctx.dir) {
    return (
      <div className="p-3 text-xs text-muted-foreground">{gt("Clone or link a folder first.")}</div>
    );
  }
  const mainDir = trees.data?.find((tree) => tree.main)?.path ?? ctx.dir;
  const base = props.currentBranch ?? ctx.defaultBranch ?? "main";
  const dir = ctx.dir;

  const run = async (key: string, request: Record<string, unknown>, ok: string) => {
    setBusyKey(key);
    const result = await callGh(request);
    setBusyKey(null);
    if (!result.ok) {
      ctx.toast("error", gt("That did not work"), result.error);
      return false;
    }
    ctx.toast("success", ok);
    ctx.onChanged();
    return true;
  };

  if (creating) {
    return (
      <NewBranchCard ctx={ctx} base={base} mainDir={mainDir} onDone={() => setCreating(false)} />
    );
  }
  if (folderFor) {
    return (
      <FormCard
        title={gt("Give {branch} its own folder", { branch: folderFor })}
        submitLabel={gt("Create and use in this chat")}
        busy={busyKey === "tree"}
        disabled={!folderPath.trim()}
        onCancel={() => setFolderFor(null)}
        onSubmit={() =>
          void run(
            "tree",
            { op: "worktreeAdd", cwd: dir, branch: folderFor, path: folderPath.trim() },
            gt("Folder created"),
          ).then((done) => {
            if (done) {
              ctx.connectClaude(folderPath.trim());
              setFolderFor(null);
            }
          })
        }
      >
        <Field label={gt("Folder")}>
          <Input
            size="compact"
            autoFocus
            value={folderPath}
            spellCheck={false}
            onChange={(event) => setFolderPath(event.target.value)}
          />
        </Field>
        <p className="text-xs text-muted-foreground">
          {gt("The branch is checked out there. Your current folder keeps its own branch.")}
        </p>
      </FormCard>
    );
  }

  return (
    <div className="flex flex-col pb-3">
      <div className="flex items-center justify-between px-3 pt-3 pb-1.5">
        <span className="text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">
          {gt("Folders for branches")}
        </span>
      </div>
      {trees.loading && !trees.data ? (
        <div className="px-3 py-2">
          <Spinner className="size-3.5" />
        </div>
      ) : null}
      <ul>
        {(trees.data ?? []).map((tree) => {
          const here = ctx.chatFolder === tree.path;
          return (
            <li key={tree.path} className="flex items-center gap-2 px-3 py-1.5">
              <div className="min-w-0 flex-1">
                <div className="truncate text-xs font-medium text-foreground">
                  {tree.branch ?? gt("(detached)")}
                </div>
                <div className="truncate text-[11px] text-muted-foreground" title={tree.path}>
                  {tidyPath(tree.path)}
                </div>
              </div>
              {here ? (
                <span className="text-[11px] text-emerald-500">{gt("Claude is here")}</span>
              ) : (
                <Button size="xs" variant="outline" onClick={() => ctx.connectClaude(tree.path)}>
                  {gt("Use in chat")}
                </Button>
              )}
              {!tree.main ? (
                <Button
                  size="icon-xs"
                  variant="ghost"
                  aria-label={gt("Remove folder")}
                  disabled={busyKey === tree.path}
                  onClick={() =>
                    void run(
                      tree.path,
                      { op: "worktreeRemove", cwd: dir, path: tree.path },
                      gt("Folder removed"),
                    )
                  }
                >
                  <Trash2Icon className="size-3.5" />
                </Button>
              ) : null}
            </li>
          );
        })}
      </ul>

      <div className="mt-2 flex items-center justify-between px-3 pt-2 pb-1.5">
        <span className="text-[11px] font-semibold tracking-wide text-muted-foreground uppercase">
          {gt("Branches")}
        </span>
        <Button size="xs" variant="outline" onClick={() => setCreating(true)}>
          <PlusIcon className="size-3" />
          {gt("New branch")}
        </Button>
      </div>
      {branches.error ? (
        <div className="px-3 text-xs text-destructive">{branches.error}</div>
      ) : null}
      <ul>
        {(branches.data ?? []).map((branch) => {
          const hasTree = (trees.data ?? []).some((tree) => tree.branch === branch.name);
          return (
            <li
              key={branch.name}
              className="flex items-start gap-2 border-b border-border/30 px-3 py-2"
            >
              <div className="min-w-0 flex-1">
                <div className="flex items-center gap-1.5 text-xs font-medium text-foreground">
                  {branch.current ? <CheckIcon className="size-3 text-emerald-500" /> : null}
                  <span className="truncate">{branch.name}</span>
                  {branch.track ? (
                    <span className="text-[10px] font-normal text-muted-foreground">
                      {branch.track}
                    </span>
                  ) : null}
                </div>
                <div className="truncate text-[11px] text-muted-foreground">
                  {branch.when} · {branch.subject}
                </div>
              </div>
              {!branch.current ? (
                <Button
                  size="xs"
                  variant="outline"
                  disabled={busyKey === branch.name}
                  onClick={() =>
                    void run(
                      branch.name,
                      { op: "switch", cwd: dir, branch: branch.name },
                      gt("Switched to {branch}", { branch: branch.name }),
                    )
                  }
                >
                  {gt("Switch")}
                </Button>
              ) : null}
              {!hasTree ? (
                <Button
                  size="icon-xs"
                  variant="ghost"
                  aria-label={gt("Own folder")}
                  title={gt("Give this branch its own folder")}
                  onClick={() => {
                    setFolderFor(branch.name);
                    setFolderPath(siblingFolder(mainDir, ctx.repo, branch.name));
                  }}
                >
                  <FolderPlusIcon className="size-3.5" />
                </Button>
              ) : null}
              {!branch.current && !hasTree ? (
                <Button
                  size="icon-xs"
                  variant="ghost"
                  aria-label={gt("Delete branch")}
                  title={gt("Delete branch (only if merged)")}
                  disabled={busyKey === `del:${branch.name}`}
                  onClick={() =>
                    void run(
                      `del:${branch.name}`,
                      { op: "deleteBranch", cwd: dir, name: branch.name },
                      gt("Deleted {branch}", { branch: branch.name }),
                    )
                  }
                >
                  <Trash2Icon className="size-3.5" />
                </Button>
              ) : null}
            </li>
          );
        })}
      </ul>
    </div>
  );
}
