// CC3: commit history of a repository branch, with the changes of one commit rendered like the Diff panel.
import { FileDiff } from "@pierre/diffs/react";
import { useGT } from "gt-react";
import { ArrowLeftIcon } from "lucide-react";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Spinner } from "~/components/ui/spinner";
import { useRenderedAppearance } from "~/hooks/useTheme";
import { getRenderablePatch, resolveDiffThemeName } from "~/lib/diffRendering";
import { PREFERRED_HIGHLIGHTER } from "~/lib/syntaxHighlighting";

import { DiffWorkerPoolProvider } from "../DiffWorkerPoolProvider";
import { DIFF_VIEW_UNSAFE_CSS } from "../diffs/StyledDiffCodeView";
import { useGh } from "./ghHooks";
import { callGh } from "./ghBridge";

interface Commit {
  sha: string;
  message: string;
  author: string;
  date: string;
}

const relativeTime = (iso: string) => {
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

function CommitDetail(props: {
  readonly repo: string;
  readonly commit: Commit;
  readonly onBack: () => void;
}) {
  const gt = useGT();
  const appearance = useRenderedAppearance();
  const detail = useGh<{ diff: string }>({ op: "commit", repo: props.repo, sha: props.commit.sha });
  const patch = detail.data
    ? getRenderablePatch(detail.data.diff, `github:${props.commit.sha}`)
    : null;
  const [title, ...body] = props.commit.message.split("\n");
  const options = {
    diffStyle: "unified" as const,
    overflow: "scroll" as const,
    theme: resolveDiffThemeName(appearance),
    themeType: appearance,
    preferredHighlighter: PREFERRED_HIGHLIGHTER,
    unsafeCSS: DIFF_VIEW_UNSAFE_CSS,
  };
  return (
    <div className="flex min-h-0 flex-col">
      <div className="flex flex-col gap-1 border-b border-border/60 px-3 py-2">
        <Button size="xs" variant="ghost" className="-ml-2 self-start" onClick={props.onBack}>
          <ArrowLeftIcon className="size-3" />
          {gt("Commits")}
        </Button>
        <div className="text-sm font-medium text-foreground">{title}</div>
        {body.join("\n").trim() ? (
          <pre className="whitespace-pre-wrap font-sans text-xs text-muted-foreground">
            {body.join("\n").trim()}
          </pre>
        ) : null}
        <div className="text-[11px] text-muted-foreground">
          {props.commit.author} · {relativeTime(props.commit.date)} · {props.commit.sha.slice(0, 7)}
        </div>
      </div>
      {detail.loading ? (
        <div className="p-3">
          <Spinner className="size-4" />
        </div>
      ) : detail.error ? (
        <div className="p-3 text-xs text-destructive">{detail.error}</div>
      ) : patch?.kind === "files" ? (
        <DiffWorkerPoolProvider>
          <div className="flex flex-col gap-3 p-2">
            {patch.files.map((file) => (
              <div
                key={file.cacheKey ?? file.name}
                className="overflow-hidden rounded-md border border-border/60"
              >
                <FileDiff fileDiff={file} options={options} />
              </div>
            ))}
          </div>
        </DiffWorkerPoolProvider>
      ) : patch?.kind === "raw" ? (
        <pre className="overflow-auto p-3 font-mono text-[11px]">{patch.text}</pre>
      ) : (
        <div className="p-3 text-xs text-muted-foreground">{gt("No file changes.")}</div>
      )}
    </div>
  );
}

export function GitHubCommits(props: {
  readonly repo: string;
  readonly branch: string;
  readonly refresh: number;
}) {
  const gt = useGT();
  const [pages, setPages] = useState(1);
  const [selected, setSelected] = useState<Commit | null>(null);
  const [extra, setExtra] = useState<Commit[]>([]);
  const [loadingMore, setLoadingMore] = useState(false);
  const [exhausted, setExhausted] = useState(false);
  const first = useGh<Commit[]>(
    { op: "commits", repo: props.repo, ref: props.branch, page: 1 },
    props.refresh,
  );
  const commits = [...(first.data ?? []), ...extra];

  const loadMore = async () => {
    setLoadingMore(true);
    const next = pages + 1;
    const result = await callGh({ op: "commits", repo: props.repo, ref: props.branch, page: next });
    setLoadingMore(false);
    if (!result.ok) return;
    const data = result.data as Commit[];
    setExtra((previous) => [...previous, ...data]);
    setPages(next);
    if (data.length < 30) setExhausted(true);
  };

  if (selected) {
    return <CommitDetail repo={props.repo} commit={selected} onBack={() => setSelected(null)} />;
  }
  if (first.loading) {
    return (
      <div className="p-3">
        <Spinner className="size-4" />
      </div>
    );
  }
  if (first.error) return <div className="p-3 text-xs text-destructive">{first.error}</div>;
  return (
    <ul className="flex flex-col">
      {commits.map((commit) => (
        <li key={commit.sha}>
          <button
            type="button"
            className="flex w-full flex-col items-start gap-0.5 border-b border-border/40 px-3 py-2 text-left hover:bg-accent/60"
            onClick={() => setSelected(commit)}
          >
            <span className="w-full truncate text-xs font-medium text-foreground">
              {commit.message.split("\n")[0]}
            </span>
            <span className="text-[11px] text-muted-foreground">
              {commit.author} · {relativeTime(commit.date)} · {commit.sha.slice(0, 7)}
            </span>
          </button>
        </li>
      ))}
      {commits.length >= 30 && !exhausted ? (
        <li className="p-2">
          <Button
            size="sm"
            variant="outline"
            className="w-full"
            disabled={loadingMore}
            onClick={() => void loadMore()}
          >
            {loadingMore ? <Spinner className="size-3.5" /> : gt("Load more")}
          </Button>
        </li>
      ) : null}
    </ul>
  );
}
