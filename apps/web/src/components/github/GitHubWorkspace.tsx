// CC3: "where Claude works" card of the GitHub panel: folder, branch, sync state and the daily git actions.
import { useGT } from "gt-react";
import { ArrowDownIcon, ArrowUpIcon, FolderIcon, GitBranchIcon } from "lucide-react";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Spinner } from "~/components/ui/spinner";

import { callGh } from "./ghBridge";
import { type Query, type WorkCtx, type WtStatus, tidyPath } from "./ghHooks";

export function WorkspaceCard(props: {
  readonly ctx: WorkCtx;
  readonly status: Query<WtStatus>;
  readonly chatHere: boolean;
  readonly leaveTarget: string | null;
  readonly onLink: () => void;
  readonly onClone: () => void;
}) {
  const gt = useGT();
  const { ctx } = props;
  const [busy, setBusy] = useState<"pull" | "push" | "fetch" | null>(null);
  const status = props.status.data;

  const act = async (kind: "pull" | "push" | "fetch") => {
    if (!ctx.dir) return;
    setBusy(kind);
    const result = await callGh({ op: kind, cwd: ctx.dir });
    setBusy(null);
    if (!result.ok) {
      ctx.toast("error", gt("Could not {action}", { action: kind }), result.error);
      return;
    }
    ctx.onChanged();
  };

  if (!ctx.dir) {
    return (
      <div className="flex flex-col gap-2 rounded-lg border border-dashed border-border/70 p-3">
        <div className="text-xs font-medium text-foreground">
          {gt("This repository is not on your computer yet")}
        </div>
        <div className="text-[11px] text-muted-foreground">
          {gt("Clone it or link a folder that already holds it. Then Claude can work on it.")}
        </div>
        <div className="flex gap-2">
          <Button size="sm" onClick={props.onClone}>
            {gt("Clone")}
          </Button>
          <Button size="sm" variant="outline" onClick={props.onLink}>
            {gt("Link folder")}
          </Button>
        </div>
      </div>
    );
  }

  const changes = status?.files.length ?? 0;
  return (
    <div className="flex flex-col gap-2 rounded-lg border border-border/70 bg-card p-3">
      <div className="flex items-center gap-2">
        <FolderIcon className="size-3.5 shrink-0 text-muted-foreground" />
        <span className="min-w-0 flex-1 truncate text-xs text-foreground" title={ctx.dir}>
          {tidyPath(ctx.dir)}
        </span>
        {props.chatHere ? (
          <>
            <span className="rounded-sm bg-emerald-500/15 px-1.5 py-0.5 text-[10px] font-medium text-emerald-500">
              {gt("Claude is here")}
            </span>
            {props.leaveTarget ? (
              <Button
                size="xs"
                variant="ghost"
                onClick={() => ctx.connectClaude(props.leaveTarget ?? "")}
              >
                {gt("Leave")}
              </Button>
            ) : null}
          </>
        ) : (
          <Button size="xs" variant="outline" onClick={() => ctx.connectClaude(ctx.dir ?? "")}>
            {gt("Use in this chat")}
          </Button>
        )}
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <GitBranchIcon className="size-3.5 shrink-0 text-muted-foreground" />
        {props.status.loading && !status ? (
          <Spinner className="size-3" />
        ) : (
          <span className="text-xs font-medium text-foreground">{status?.branch ?? "—"}</span>
        )}
        {status?.upstream ? (
          <span className="flex items-center gap-1.5 text-[11px] text-muted-foreground">
            <span className="flex items-center">
              <ArrowUpIcon className="size-3" />
              {status.ahead}
            </span>
            <span className="flex items-center">
              <ArrowDownIcon className="size-3" />
              {status.behind}
            </span>
          </span>
        ) : status ? (
          <span className="text-[11px] text-muted-foreground">{gt("not published")}</span>
        ) : null}
        <span className="flex-1" />
        <span className="text-[11px] text-muted-foreground">
          {changes ? gt("{count} changes", { count: changes }) : gt("clean")}
        </span>
      </div>
      <div className="flex items-center gap-2">
        <Button
          size="xs"
          variant="outline"
          disabled={busy !== null}
          onClick={() => void act("fetch")}
        >
          {busy === "fetch" ? <Spinner className="size-3" /> : gt("Fetch")}
        </Button>
        <Button
          size="xs"
          variant={status && status.behind > 0 ? "default" : "outline"}
          disabled={busy !== null || !status?.upstream}
          onClick={() => void act("pull")}
        >
          {busy === "pull" ? <Spinner className="size-3" /> : gt("Pull")}
        </Button>
        <Button
          size="xs"
          variant={status && (status.ahead > 0 || !status.upstream) ? "default" : "outline"}
          disabled={busy !== null || !status || (status.upstream !== null && status.ahead === 0)}
          onClick={() => void act("push")}
        >
          {busy === "push" ? (
            <Spinner className="size-3" />
          ) : status?.upstream ? (
            gt("Push")
          ) : (
            gt("Publish branch")
          )}
        </Button>
      </div>
    </div>
  );
}
