// CC3: the top of the thread details card. Repository and branch, the working folder, the context
// window and background tasks, each row clickable and brightening on hover.
import { scopeThreadRef } from "@t3tools/client-runtime/environment";
import type { ContextWindowSnapshot } from "@t3tools/client-runtime/state/context-window";
import type { EnvironmentId, OrchestrationV2PendingBackgroundTask } from "@t3tools/contracts";
import { useNavigate } from "@tanstack/react-router";
import { useGT } from "gt-react";
import { CpuIcon, FolderIcon, GaugeIcon, GitBranchIcon, PlugIcon } from "lucide-react";
import type { ReactNode } from "react";

import { openCommandPalette } from "../../commandPaletteBus";
import { formatContextWindowTokens } from "../../lib/contextWindow";
import { cn } from "../../lib/utils";
import { buildThreadRouteParams } from "../../threadRoutes";
import { ThreadDetailsSection } from "./ThreadDetailsSection";

const ROW =
  "flex min-h-8 w-full min-w-0 cursor-pointer items-center gap-2.5 rounded-lg px-2.5 text-left text-[13px] font-medium text-foreground/80 transition-colors duration-150 hover:bg-black/[0.035] hover:text-foreground dark:hover:bg-white/[0.08]";

function Row(props: { icon: ReactNode; onClick: () => void; title?: string; children: ReactNode }) {
  return (
    <button type="button" title={props.title} onClick={props.onClick} className={ROW}>
      <span className="shrink-0 text-muted-foreground [&_svg]:size-4">{props.icon}</span>
      {props.children}
    </button>
  );
}

function Stat(props: { label: string; value: string; percent?: number; onClick: () => void }) {
  return (
    <button
      type="button"
      onClick={props.onClick}
      className={cn(ROW, "min-h-7 gap-2 py-0.5 text-xs font-normal text-muted-foreground")}
    >
      <span className="min-w-0 flex-1 truncate">{props.label}</span>
      <span className="tabular-nums text-foreground/80">{props.value}</span>
      {props.percent !== undefined ? (
        <span className="w-12 text-right tabular-nums">{props.percent.toFixed(1)}%</span>
      ) : null}
    </button>
  );
}

export interface ThreadDetailsOverviewInput {
  readonly environmentId: EnvironmentId;
  readonly repository: string | null;
  readonly branch: string | null;
  readonly directory: string | null;
  readonly onPickDirectory: () => void;
  readonly contextWindow: ContextWindowSnapshot | null;
  readonly backgroundTasks: ReadonlyArray<OrchestrationV2PendingBackgroundTask>;
}

export function ThreadDetailsOverview(props: ThreadDetailsOverviewInput) {
  const gt = useGT();
  const navigate = useNavigate();
  const openUsage = () => void navigate({ to: "/usage" });
  const context = props.contextWindow;
  const max = context?.maxTokens != null && context.maxTokens > 0 ? context.maxTokens : null;
  const used = context?.usedTokens ?? 0;
  const usedPercent = max !== null ? Math.min(100, Math.max(0, (used / max) * 100)) : null;
  const threshold = context?.autoCompactThreshold ?? null;
  const buffer = max !== null && threshold !== null && threshold < max ? max - threshold : null;
  const free = max !== null ? Math.max(0, max - used - (buffer ?? 0)) : null;
  const pct = (tokens: number) => (max !== null ? (tokens / max) * 100 : undefined);
  const categories =
    context?.contextCategories && context.contextCategories.length > 0
      ? [...context.contextCategories].toSorted((a, b) => {
          const order = { used: 0, buffer: 1, free: 2, deferred: 3 } as const;
          return order[a.kind] - order[b.kind] || b.tokens - a.tokens;
        })
      : null;
  const servers = context?.mcpServers ?? [];
  const home = (path: string) => path.replace(/^\/home\/[^/]+/, "~");

  return (
    <>
      <ThreadDetailsSection
        headingId="cc3-details-repo-heading"
        title={gt("Repository")}
        showHeading={false}
        separated={false}
      >
        <div className="flex flex-col">
          <Row
            icon={<GitBranchIcon />}
            title={gt("Add a project or create a GitHub repository")}
            onClick={() => openCommandPalette({ open: "add-project" })}
          >
            <span className="min-w-0 truncate">
              {props.repository
                ? `${props.repository}${props.branch ? ` / ${props.branch}` : ""}`
                : gt("No repo")}
            </span>
          </Row>
          {props.directory ? (
            <Row
              icon={<FolderIcon />}
              title={gt("Choose a folder (starts a new chat there)")}
              onClick={props.onPickDirectory}
            >
              <span className="min-w-0 truncate">{home(props.directory)}</span>
            </Row>
          ) : null}
        </div>
      </ThreadDetailsSection>

      {context ? (
        <ThreadDetailsSection
          headingId="cc3-details-context-heading"
          title={gt("Context")}
          showHeading={false}
        >
          <div className="flex flex-col gap-1">
            <button
              type="button"
              onClick={openUsage}
              className={cn(ROW, "flex-col items-stretch gap-1.5 py-2")}
            >
              <span className="flex items-baseline justify-between gap-2">
                <span className="flex items-center gap-1.5">
                  <GaugeIcon className="size-4 text-muted-foreground" aria-hidden />
                  {usedPercent !== null
                    ? gt("{percent}% used", { percent: Math.round(usedPercent) })
                    : gt("Context")}
                </span>
                <span className="text-xs font-normal text-muted-foreground tabular-nums">
                  {formatContextWindowTokens(used)}
                  {max !== null ? ` / ${formatContextWindowTokens(max)}` : ""}
                </span>
              </span>
              {usedPercent !== null ? (
                <span className="relative block h-1.5 w-full overflow-hidden rounded-full bg-muted">
                  <span
                    className="absolute inset-y-0 left-0 rounded-full bg-primary"
                    style={{ width: `${usedPercent}%` }}
                  />
                  {buffer !== null && max !== null ? (
                    <span
                      className="absolute inset-y-0 right-0 bg-foreground/25"
                      style={{ width: `${(buffer / max) * 100}%` }}
                    />
                  ) : null}
                </span>
              ) : null}
            </button>
            {categories
              ? categories.map((category) => (
                  <Stat
                    key={`${category.id ?? category.name}`}
                    label={category.name}
                    value={
                      category.kind === "deferred"
                        ? "\u2014"
                        : formatContextWindowTokens(category.tokens)
                    }
                    {...(category.kind !== "deferred" && pct(category.tokens) !== undefined
                      ? { percent: pct(category.tokens)! }
                      : {})}
                    onClick={openUsage}
                  />
                ))
              : null}
            {categories ? null : (
              <Stat
                label={gt("Used")}
                value={formatContextWindowTokens(used)}
                {...(pct(used) !== undefined ? { percent: pct(used)! } : {})}
                onClick={openUsage}
              />
            )}
            {categories ? null : buffer !== null ? (
              <Stat
                label={gt("Autocompact buffer")}
                value={formatContextWindowTokens(buffer)}
                {...(pct(buffer) !== undefined ? { percent: pct(buffer)! } : {})}
                onClick={openUsage}
              />
            ) : null}
            {categories ? null : free !== null ? (
              <Stat
                label={gt("Free space")}
                value={formatContextWindowTokens(free)}
                {...(pct(free) !== undefined ? { percent: pct(free)! } : {})}
                onClick={openUsage}
              />
            ) : null}
          </div>
        </ThreadDetailsSection>
      ) : null}

      {servers.length > 0 ? (
        <ThreadDetailsSection
          headingId="cc3-details-connectors-heading"
          title={gt("Connectors")}
          showHeading
        >
          <div className="flex flex-col">
            {servers.map((server) => (
              <Row
                key={server.name}
                icon={<PlugIcon />}
                title={server.status}
                onClick={() => void navigate({ to: "/settings/integrations" })}
              >
                <span className="min-w-0 truncate">{server.name}</span>
                <span
                  aria-label={server.status}
                  className={cn(
                    "ms-auto size-2 shrink-0 rounded-full",
                    server.status === "connected"
                      ? "bg-success"
                      : server.status === "failed"
                        ? "bg-destructive"
                        : server.status === "needs-auth"
                          ? "bg-warning"
                          : "bg-muted-foreground/50",
                  )}
                />
              </Row>
            ))}
          </div>
        </ThreadDetailsSection>
      ) : null}

      <ThreadDetailsSection
        headingId="cc3-details-tasks-heading"
        title={gt("Background tasks")}
        showHeading
      >
        <div className="flex flex-col">
          {props.backgroundTasks.length === 0 ? (
            <p className="px-2.5 py-1 text-xs text-muted-foreground">{gt("None running")}</p>
          ) : (
            props.backgroundTasks.map((task) => (
              <Row
                key={task.taskId}
                icon={<CpuIcon />}
                onClick={() => {
                  if (task.kind === "subagent" && task.childThreadId) {
                    void navigate({
                      to: "/$environmentId/$threadId",
                      params: buildThreadRouteParams(
                        scopeThreadRef(props.environmentId, task.childThreadId),
                      ),
                    });
                  }
                }}
              >
                <span className="min-w-0 truncate">{task.description ?? task.kind}</span>
                <span className="ms-auto shrink-0 text-xs font-normal text-muted-foreground">
                  {task.kind}
                </span>
              </Row>
            ))
          )}
        </div>
      </ThreadDetailsSection>
    </>
  );
}
