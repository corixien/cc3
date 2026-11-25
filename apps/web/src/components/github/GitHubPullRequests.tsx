// CC3: pull requests of a repository: list with CI and review state, open one for the current branch,
// check one out into the folder Claude works in, and merge.
import { useGT } from "gt-react";
import { ExternalLinkIcon, GitPullRequestIcon, SparklesIcon } from "lucide-react";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Checkbox } from "~/components/ui/checkbox";
import { Input } from "~/components/ui/input";
import { Spinner } from "~/components/ui/spinner";
import { Textarea } from "~/components/ui/textarea";
import { readLocalApi } from "~/localApi";

import { callGh } from "./ghBridge";
import { type WorkCtx, relativeTime, useGh } from "./ghHooks";
import { Field, FormCard, SimpleSelect } from "./GitHubUi";

interface PullRequest {
  number: number;
  title: string;
  state: "OPEN" | "CLOSED" | "MERGED";
  isDraft: boolean;
  headRefName: string;
  baseRefName: string;
  author: { login: string } | null;
  url: string;
  reviewDecision: string | null;
  statusCheckRollup: Array<{ conclusion?: string; status?: string; state?: string }> | null;
  updatedAt: string;
}

const checks = (pr: PullRequest): "pass" | "fail" | "pending" | null => {
  const list = pr.statusCheckRollup ?? [];
  if (!list.length) return null;
  const bad = (value?: string) =>
    ["FAILURE", "ERROR", "TIMED_OUT", "CANCELLED"].includes(value ?? "");
  if (list.some((entry) => bad(entry.conclusion) || bad(entry.state))) return "fail";
  if (
    list.some(
      (entry) => (entry.status && entry.status !== "COMPLETED") || entry.state === "PENDING",
    )
  ) {
    return "pending";
  }
  return "pass";
};

function Chip(props: {
  readonly tone: "green" | "red" | "amber" | "muted" | "purple";
  readonly children: string;
}) {
  const tones = {
    green: "bg-emerald-500/15 text-emerald-500",
    red: "bg-red-500/15 text-red-500",
    amber: "bg-amber-500/15 text-amber-500",
    purple: "bg-violet-500/15 text-violet-400",
    muted: "bg-muted text-muted-foreground",
  } as const;
  return (
    <span className={`rounded-sm px-1.5 py-0.5 text-[10px] font-medium ${tones[props.tone]}`}>
      {props.children}
    </span>
  );
}

function NewPullRequest(props: {
  readonly ctx: WorkCtx;
  readonly branch: string;
  readonly onDone: () => void;
}) {
  const gt = useGT();
  const { ctx } = props;
  const [title, setTitle] = useState(props.branch.replace(/[-_/]+/g, " "));
  const [body, setBody] = useState("");
  const [base, setBase] = useState(ctx.defaultBranch ?? "main");
  const [draft, setDraft] = useState(false);
  const [busy, setBusy] = useState(false);
  const branches = useGh<string[]>({ op: "branches", repo: ctx.repo });
  const submit = async () => {
    if (!ctx.dir) return;
    setBusy(true);
    const result = await callGh({ op: "prCreate", cwd: ctx.dir, title, body, base, draft });
    setBusy(false);
    if (!result.ok) {
      ctx.toast("error", gt("Could not open the pull request"), result.error);
      return;
    }
    const url = (result.data as { url: string }).url;
    ctx.toast("success", gt("Pull request opened"), url);
    ctx.onChanged();
    props.onDone();
  };
  return (
    <FormCard
      title={gt("Open a pull request for {branch}", { branch: props.branch })}
      submitLabel={gt("Open pull request")}
      busy={busy}
      disabled={!title.trim() || base === props.branch}
      onCancel={props.onDone}
      onSubmit={() => void submit()}
      extraAction={
        <Button
          type="button"
          size="sm"
          variant="outline"
          onClick={() => {
            ctx.askClaude(
              gt(
                "Open a pull request for this branch into {base}. Write a clear title and description from the commits.",
                { base },
              ).toString(),
            );
            props.onDone();
          }}
        >
          <SparklesIcon className="size-3.5" />
          {gt("Let Claude write it")}
        </Button>
      }
    >
      <Field label={gt("Title")}>
        <Input
          size="compact"
          autoFocus
          value={title}
          onChange={(event) => setTitle(event.target.value)}
        />
      </Field>
      <Field label={gt("Description")}>
        <Textarea
          size="sm"
          rows={4}
          value={body}
          onChange={(event) => setBody(event.target.value)}
        />
      </Field>
      <Field label={gt("Merge into")}>
        <SimpleSelect
          ariaLabel={gt("Base branch")}
          value={base}
          onChange={setBase}
          items={(branches.data ?? [base]).map((name) => ({ value: name, label: name }))}
        />
      </Field>
      <label className="flex items-center gap-2 text-xs text-muted-foreground">
        <Checkbox checked={draft} onCheckedChange={(checked) => setDraft(checked === true)} />
        {gt("Draft (not ready for review yet)")}
      </label>
    </FormCard>
  );
}

function MergeCard(props: {
  readonly ctx: WorkCtx;
  readonly pr: PullRequest;
  readonly onDone: () => void;
}) {
  const gt = useGT();
  const { ctx, pr } = props;
  const [method, setMethod] = useState("squash");
  const [deleteBranch, setDeleteBranch] = useState(true);
  const [busy, setBusy] = useState(false);
  const submit = async () => {
    setBusy(true);
    const result = await callGh({
      op: "prMerge",
      repo: ctx.repo,
      number: pr.number,
      method,
      deleteBranch,
    });
    setBusy(false);
    if (!result.ok) {
      ctx.toast("error", gt("Could not merge"), result.error);
      return;
    }
    ctx.toast("success", gt("Merged #{number}", { number: pr.number }));
    ctx.onChanged();
    props.onDone();
  };
  return (
    <FormCard
      title={gt("Merge #{number} {title}", { number: pr.number, title: pr.title })}
      submitLabel={gt("Merge")}
      busy={busy}
      onCancel={props.onDone}
      onSubmit={() => void submit()}
    >
      <Field label={gt("Method")}>
        <SimpleSelect
          ariaLabel={gt("Merge method")}
          value={method}
          onChange={setMethod}
          items={[
            { value: "squash", label: gt("Squash (one commit)") },
            { value: "merge", label: gt("Merge commit") },
            { value: "rebase", label: gt("Rebase") },
          ]}
        />
      </Field>
      <label className="flex items-center gap-2 text-xs text-muted-foreground">
        <Checkbox
          checked={deleteBranch}
          onCheckedChange={(checked) => setDeleteBranch(checked === true)}
        />
        {gt("Delete the branch on GitHub afterwards")}
      </label>
    </FormCard>
  );
}

export function GitHubPullRequests(props: {
  readonly ctx: WorkCtx;
  readonly currentBranch: string | null;
}) {
  const gt = useGT();
  const { ctx } = props;
  const [state, setState] = useState("open");
  const [mode, setMode] = useState<"list" | "new" | { merge: PullRequest }>("list");
  const [busyNumber, setBusyNumber] = useState<number | null>(null);
  const prs = useGh<PullRequest[]>({ op: "prs", repo: ctx.repo, state }, ctx.revision);

  if (mode === "new" && props.currentBranch) {
    return <NewPullRequest ctx={ctx} branch={props.currentBranch} onDone={() => setMode("list")} />;
  }
  if (typeof mode === "object") {
    return <MergeCard ctx={ctx} pr={mode.merge} onDone={() => setMode("list")} />;
  }

  const checkout = async (number: number) => {
    if (!ctx.dir) return;
    setBusyNumber(number);
    const result = await callGh({ op: "prCheckout", cwd: ctx.dir, number });
    setBusyNumber(null);
    if (!result.ok) {
      ctx.toast("error", gt("Could not check out the pull request"), result.error);
      return;
    }
    ctx.toast("success", gt("Checked out #{number}", { number }));
    ctx.onChanged();
  };

  const canOpenNew =
    ctx.dir !== null && props.currentBranch !== null && props.currentBranch !== ctx.defaultBranch;
  return (
    <div className="flex flex-col">
      <div className="flex items-center gap-2 border-b border-border/60 px-3 py-2">
        <SimpleSelect
          ariaLabel={gt("State")}
          value={state}
          onChange={setState}
          items={[
            { value: "open", label: gt("Open") },
            { value: "merged", label: gt("Merged") },
            { value: "closed", label: gt("Closed") },
            { value: "all", label: gt("All") },
          ]}
          className="w-32 min-w-0"
        />
        <span className="flex-1" />
        <Button
          size="sm"
          variant={canOpenNew ? "default" : "outline"}
          disabled={!canOpenNew}
          onClick={() => setMode("new")}
          title={canOpenNew ? "" : gt("Switch to a feature branch first")}
        >
          <GitPullRequestIcon className="size-3.5" />
          {gt("New pull request")}
        </Button>
      </div>
      {prs.loading && !prs.data ? (
        <div className="p-3">
          <Spinner className="size-4" />
        </div>
      ) : null}
      {prs.error ? <div className="p-3 text-xs text-destructive">{prs.error}</div> : null}
      {prs.data && !prs.data.length ? (
        <div className="p-3 text-xs text-muted-foreground">{gt("No pull requests here.")}</div>
      ) : null}
      <ul>
        {(prs.data ?? []).map((pr) => {
          const ci = checks(pr);
          return (
            <li
              key={pr.number}
              className="flex flex-col gap-1.5 border-b border-border/40 px-3 py-2.5"
            >
              <div className="flex items-start gap-2">
                <span className="min-w-0 flex-1 text-xs font-medium text-foreground">
                  <span className="text-muted-foreground">#{pr.number}</span> {pr.title}
                </span>
              </div>
              <div className="flex flex-wrap items-center gap-1.5">
                <Chip
                  tone={
                    pr.state === "OPEN"
                      ? pr.isDraft
                        ? "muted"
                        : "green"
                      : pr.state === "MERGED"
                        ? "purple"
                        : "red"
                  }
                >
                  {pr.state === "OPEN"
                    ? pr.isDraft
                      ? gt("Draft")
                      : gt("Open")
                    : pr.state === "MERGED"
                      ? gt("Merged")
                      : gt("Closed")}
                </Chip>
                {ci ? (
                  <Chip tone={ci === "pass" ? "green" : ci === "fail" ? "red" : "amber"}>
                    {ci === "pass"
                      ? gt("Checks passed")
                      : ci === "fail"
                        ? gt("Checks failed")
                        : gt("Checks running")}
                  </Chip>
                ) : null}
                {pr.reviewDecision === "APPROVED" ? (
                  <Chip tone="green">{gt("Approved")}</Chip>
                ) : null}
                {pr.reviewDecision === "CHANGES_REQUESTED" ? (
                  <Chip tone="amber">{gt("Changes requested")}</Chip>
                ) : null}
                <span className="text-[11px] text-muted-foreground">
                  {pr.headRefName} → {pr.baseRefName} · {pr.author?.login ?? ""} ·{" "}
                  {relativeTime(pr.updatedAt)}
                </span>
              </div>
              <div className="flex items-center gap-2">
                {pr.state === "OPEN" && ctx.dir ? (
                  <Button
                    size="xs"
                    variant="outline"
                    disabled={busyNumber === pr.number}
                    onClick={() => void checkout(pr.number)}
                  >
                    {gt("Check out")}
                  </Button>
                ) : null}
                {pr.state === "OPEN" ? (
                  <Button
                    size="xs"
                    variant="outline"
                    disabled={pr.isDraft}
                    onClick={() => setMode({ merge: pr })}
                  >
                    {gt("Merge…")}
                  </Button>
                ) : null}
                <Button
                  size="xs"
                  variant="ghost"
                  onClick={() => void readLocalApi()?.shell.openExternal(pr.url)}
                >
                  <ExternalLinkIcon className="size-3" />
                  {gt("GitHub")}
                </Button>
              </div>
            </li>
          );
        })}
      </ul>
    </div>
  );
}
