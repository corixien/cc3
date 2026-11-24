// CC3: pull selected files of a repository folder into a local folder. Existing code is never overwritten
// by default: only files that are new in the folder are preselected, changed ones are opt-in.
import { useGT } from "gt-react";
import { ArrowLeftIcon } from "lucide-react";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Spinner } from "~/components/ui/spinner";
import { toastManager } from "~/components/ui/toast";

import { callGh } from "./ghBridge";

type Status = "new" | "same" | "changed";
interface PlanFile {
  rel: string;
  size: number;
  status: Status;
}

const INPUT_CLASS =
  "h-7 min-w-0 flex-1 rounded-md border border-border/70 bg-background px-2 text-xs text-foreground outline-none focus:ring-1 focus:ring-ring";

const BADGE: Record<Status, string> = {
  new: "text-emerald-500",
  changed: "text-amber-500",
  same: "text-muted-foreground",
};

export function Cc3GitHubPull(props: {
  readonly repo: string;
  readonly branch: string;
  readonly source: string;
  readonly defaultTarget: string;
  readonly currentChatFolder: string | null;
  readonly linkedFolder: string | null;
  readonly onLink: (folder: string) => void;
  readonly onClose: () => void;
  readonly onPulled: (target: string, moveClaude: boolean) => void;
}) {
  const gt = useGT();
  const [target, setTarget] = useState(props.defaultTarget);
  const [loading, setLoading] = useState(false);
  const [pulling, setPulling] = useState(false);
  const [plan, setPlan] = useState<{ files: PlanFile[]; truncated: boolean } | null>(null);
  const [selected, setSelected] = useState<ReadonlySet<string>>(new Set());
  const [filter, setFilter] = useState("");
  const [moveClaude, setMoveClaude] = useState(true);
  const [linkIt, setLinkIt] = useState(props.linkedFolder === null);

  const loadPlan = async () => {
    setLoading(true);
    const result = await callGh({
      op: "importPlan",
      repo: props.repo,
      ref: props.branch,
      source: props.source,
      cwd: target.trim(),
    });
    setLoading(false);
    if (!result.ok) {
      setPlan(null);
      toastManager.add({
        type: "error",
        title: gt("Could not read the comparison"),
        description: result.error,
      });
      return;
    }
    const data = result.data as { files: PlanFile[]; truncated: boolean };
    setPlan(data);
    setSelected(
      new Set(data.files.filter((file) => file.status === "new").map((file) => file.rel)),
    );
  };

  const pull = async () => {
    setPulling(true);
    const result = await callGh({
      op: "importFiles",
      repo: props.repo,
      ref: props.branch,
      source: props.source,
      cwd: target.trim(),
      paths: [...selected],
    });
    setPulling(false);
    if (!result.ok) {
      toastManager.add({ type: "error", title: gt("Pull failed"), description: result.error });
      return;
    }
    toastManager.add({
      type: "success",
      title: gt("Pulled {count} files", { count: selected.size }),
      description: target.trim(),
    });
    if (linkIt && target.trim() !== props.linkedFolder) props.onLink(target.trim());
    props.onPulled(target.trim(), moveClaude && target.trim() !== props.currentChatFolder);
  };

  const visible =
    plan?.files.filter(
      (file) => !filter || file.rel.toLowerCase().includes(filter.toLowerCase()),
    ) ?? [];
  const count = (status: Status) =>
    plan?.files.filter((file) => file.status === status).length ?? 0;
  const toggle = (rel: string) =>
    setSelected((previous) => {
      const next = new Set(previous);
      if (!next.delete(rel)) next.add(rel);
      return next;
    });
  const select = (statuses: readonly Status[]) =>
    setSelected(
      new Set(plan?.files.filter((file) => statuses.includes(file.status)).map((file) => file.rel)),
    );

  return (
    <div className="flex min-h-0 flex-1 flex-col">
      <div className="flex flex-col gap-1.5 border-b border-border/60 p-2">
        <div className="flex items-center gap-1">
          <Button size="icon-xs" variant="ghost" aria-label={gt("Back")} onClick={props.onClose}>
            <ArrowLeftIcon className="size-3.5" />
          </Button>
          <span className="min-w-0 flex-1 truncate text-foreground">
            {gt("Pull {source} from {repo}@{branch}", {
              source: props.source || gt("everything"),
              repo: props.repo,
              branch: props.branch,
            })}
          </span>
        </div>
        <div className="flex flex-wrap items-center gap-1.5">
          {props.linkedFolder ? (
            <Button size="xs" variant="outline" onClick={() => setTarget(props.linkedFolder ?? "")}>
              {gt("Linked folder")}
            </Button>
          ) : null}
          {props.currentChatFolder ? (
            <Button
              size="xs"
              variant="outline"
              onClick={() => setTarget(props.currentChatFolder ?? "")}
            >
              {gt("This chat's folder")}
            </Button>
          ) : null}
        </div>
        <div className="flex items-center gap-2">
          <input
            className={INPUT_CLASS}
            value={target}
            spellCheck={false}
            aria-label={gt("Into folder")}
            onChange={(event) => setTarget(event.target.value)}
            onKeyDown={(event) => {
              if (event.key === "Enter") void loadPlan();
            }}
          />
          <Button size="xs" disabled={loading || !target.trim()} onClick={() => void loadPlan()}>
            {loading ? <Spinner className="size-3" /> : gt("Compare")}
          </Button>
        </div>
      </div>
      {plan ? (
        <>
          <div className="flex flex-wrap items-center gap-1.5 border-b border-border/60 px-2 py-1.5">
            <span className="text-muted-foreground">
              {gt("{newCount} new, {changed} differ, {same} identical", {
                newCount: count("new"),
                changed: count("changed"),
                same: count("same"),
              })}
            </span>
            <Button size="xs" variant="ghost" onClick={() => select(["new"])}>
              {gt("Only new")}
            </Button>
            <Button size="xs" variant="ghost" onClick={() => select(["new", "changed"])}>
              {gt("New and differing")}
            </Button>
            <Button size="xs" variant="ghost" onClick={() => setSelected(new Set())}>
              {gt("None")}
            </Button>
            <input
              className={INPUT_CLASS}
              value={filter}
              placeholder={gt("Filter")}
              aria-label={gt("Filter files")}
              onChange={(event) => setFilter(event.target.value)}
            />
          </div>
          <ul className="min-h-0 flex-1 overflow-auto px-1 py-1">
            {visible.slice(0, 500).map((file) => (
              <li key={file.rel}>
                <label className="flex cursor-pointer items-center gap-2 rounded-sm px-2 py-0.5 hover:bg-accent">
                  <input
                    type="checkbox"
                    checked={selected.has(file.rel)}
                    disabled={file.status === "same"}
                    onChange={() => toggle(file.rel)}
                  />
                  <span className="min-w-0 flex-1 truncate text-foreground">{file.rel}</span>
                  <span className={BADGE[file.status]}>
                    {file.status === "new"
                      ? gt("new")
                      : file.status === "changed"
                        ? gt("differs")
                        : gt("same")}
                  </span>
                </label>
              </li>
            ))}
            {visible.length > 500 ? (
              <li className="px-2 py-1 text-muted-foreground">
                {gt("{count} more. Narrow the list with the filter.", {
                  count: visible.length - 500,
                })}
              </li>
            ) : null}
            {plan.truncated ? (
              <li className="px-2 py-1 text-muted-foreground">
                {gt("The repository is too large to list completely.")}
              </li>
            ) : null}
          </ul>
          <div className="flex flex-wrap items-center gap-2 border-t border-border/60 p-2">
            {target.trim() !== props.linkedFolder ? (
              <label className="flex items-center gap-1.5 text-muted-foreground">
                <input
                  type="checkbox"
                  checked={linkIt}
                  onChange={(event) => setLinkIt(event.target.checked)}
                />
                {gt("Link this folder to the repository")}
              </label>
            ) : null}
            {target.trim() !== props.currentChatFolder ? (
              <label className="flex items-center gap-1.5 text-muted-foreground">
                <input
                  type="checkbox"
                  checked={moveClaude}
                  onChange={(event) => setMoveClaude(event.target.checked)}
                />
                {gt("Move Claude to this folder")}
              </label>
            ) : null}
            <span className="flex-1" />
            <Button size="xs" disabled={pulling || selected.size === 0} onClick={() => void pull()}>
              {pulling ? (
                <Spinner className="size-3" />
              ) : (
                gt("Pull {count} files", { count: selected.size })
              )}
            </Button>
          </div>
        </>
      ) : (
        <div className="p-3 text-muted-foreground">
          {gt(
            "Choose the folder and compare. Files you already have stay untouched unless you select them.",
          )}
        </div>
      )}
    </div>
  );
}
