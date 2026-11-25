// CC3: working tree of the checkout Claude works in: changed files with their diff, and a commit form.
import { useGT } from "gt-react";
import { ChevronDownIcon, ChevronRightIcon, SparklesIcon } from "lucide-react";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Checkbox } from "~/components/ui/checkbox";
import { Spinner } from "~/components/ui/spinner";
import { Textarea } from "~/components/ui/textarea";

import { callGh } from "./ghBridge";
import { type Query, type WorkCtx, type WtStatus, useGh } from "./ghHooks";
import { PatchFiles } from "./GitHubDiff";

const BADGE = (x: string, y: string) => {
  if (x === "?") return { letter: "U", className: "text-emerald-500" };
  const code = y !== " " ? y : x;
  if (code === "D") return { letter: "D", className: "text-red-500" };
  if (code === "A") return { letter: "A", className: "text-emerald-500" };
  if (code === "R") return { letter: "R", className: "text-sky-500" };
  return { letter: code, className: "text-amber-500" };
};

function FileChange(props: {
  readonly ctx: WorkCtx;
  readonly file: { path: string; x: string; y: string };
  readonly checked: boolean;
  readonly onToggle: () => void;
}) {
  const [open, setOpen] = useState(false);
  const badge = BADGE(props.file.x, props.file.y);
  const diff = useGh<{ diff: string }>(
    open && props.ctx.dir
      ? { op: "wtDiff", cwd: props.ctx.dir, path: props.file.path, untracked: props.file.x === "?" }
      : null,
    props.ctx.revision,
  );
  return (
    <li className="border-b border-border/40">
      <div className="flex items-center gap-2 px-3 py-1.5">
        <Checkbox checked={props.checked} onCheckedChange={props.onToggle} />
        <button
          type="button"
          className="flex min-w-0 flex-1 items-center gap-1.5 text-left"
          onClick={() => setOpen((value) => !value)}
        >
          {open ? (
            <ChevronDownIcon className="size-3 shrink-0 text-muted-foreground" />
          ) : (
            <ChevronRightIcon className="size-3 shrink-0 text-muted-foreground" />
          )}
          <span className="truncate text-xs text-foreground" title={props.file.path}>
            {props.file.path}
          </span>
        </button>
        <span className={`text-[11px] font-semibold ${badge.className}`}>{badge.letter}</span>
      </div>
      {open ? (
        diff.loading ? (
          <div className="px-3 pb-2">
            <Spinner className="size-3.5" />
          </div>
        ) : diff.error ? (
          <div className="px-3 pb-2 text-[11px] text-destructive">{diff.error}</div>
        ) : diff.data ? (
          <PatchFiles
            patch={diff.data.diff}
            scope={`wt:${props.file.path}:${props.ctx.revision}`}
          />
        ) : null
      ) : null}
    </li>
  );
}

export function GitHubChanges(props: { readonly ctx: WorkCtx; readonly status: Query<WtStatus> }) {
  const gt = useGT();
  const { ctx } = props;
  const files = props.status.data?.files ?? [];
  const [unchecked, setUnchecked] = useState<ReadonlySet<string>>(new Set());
  const [message, setMessage] = useState("");
  const [busy, setBusy] = useState(false);
  const selected = files.filter((file) => !unchecked.has(file.path));

  const commit = async (push: boolean) => {
    if (!ctx.dir || !message.trim() || selected.length === 0) return;
    setBusy(true);
    const result = await callGh({
      op: "wtCommit",
      cwd: ctx.dir,
      message,
      paths: selected.map((file) => file.path),
      push,
    });
    setBusy(false);
    if (!result.ok) {
      ctx.toast("error", gt("Commit failed"), result.error);
      ctx.onChanged();
      return;
    }
    ctx.toast("success", push ? gt("Committed and pushed") : gt("Committed"));
    setMessage("");
    setUnchecked(new Set());
    ctx.onChanged();
  };

  if (!ctx.dir) {
    return (
      <div className="p-3 text-xs text-muted-foreground">{gt("Clone or link a folder first.")}</div>
    );
  }
  if (props.status.error)
    return <div className="p-3 text-xs text-destructive">{props.status.error}</div>;
  if (!files.length) {
    return (
      <div className="flex flex-col items-start gap-2 p-3 text-xs text-muted-foreground">
        <span>{gt("No changes. Everything is committed.")}</span>
        <Button
          size="xs"
          variant="outline"
          onClick={() => ctx.askClaude(gt("Make this change: ").toString())}
        >
          <SparklesIcon className="size-3" />
          {gt("Ask Claude to make a change")}
        </Button>
      </div>
    );
  }

  const toggle = (path: string) =>
    setUnchecked((previous) => {
      const next = new Set(previous);
      if (!next.delete(path)) next.add(path);
      return next;
    });
  return (
    <div className="flex min-h-0 flex-col">
      <div className="flex items-center gap-2 border-b border-border/60 px-3 py-1.5 text-[11px] text-muted-foreground">
        <Checkbox
          checked={selected.length === files.length}
          onCheckedChange={(checked) =>
            setUnchecked(checked === true ? new Set() : new Set(files.map((file) => file.path)))
          }
        />
        <span className="flex-1">
          {gt("{selected} of {total} files selected", {
            selected: selected.length,
            total: files.length,
          })}
        </span>
      </div>
      <ul>
        {files.map((file) => (
          <FileChange
            key={file.path}
            ctx={ctx}
            file={file}
            checked={!unchecked.has(file.path)}
            onToggle={() => toggle(file.path)}
          />
        ))}
      </ul>
      <div className="sticky bottom-0 flex flex-col gap-2 border-t border-border/60 bg-background p-3">
        <Textarea
          size="sm"
          rows={2}
          value={message}
          placeholder={gt("Commit message")}
          aria-label={gt("Commit message")}
          onChange={(event) => setMessage(event.target.value)}
        />
        <div className="flex flex-wrap items-center gap-2">
          <Button
            size="sm"
            disabled={busy || !message.trim() || selected.length === 0}
            onClick={() => void commit(false)}
          >
            {busy ? <Spinner className="size-3.5" /> : gt("Commit")}
          </Button>
          <Button
            size="sm"
            variant="outline"
            disabled={busy || !message.trim() || selected.length === 0}
            onClick={() => void commit(true)}
          >
            {gt("Commit and push")}
          </Button>
          <span className="flex-1" />
          <Button
            size="xs"
            variant="ghost"
            onClick={() =>
              ctx.askClaude(
                gt(
                  "Look at my uncommitted changes, write a good commit message and commit them.",
                ).toString(),
              )
            }
          >
            <SparklesIcon className="size-3" />
            {gt("Let Claude commit")}
          </Button>
        </div>
      </div>
    </div>
  );
}
