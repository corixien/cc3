// CC3: one compact line under the composer: repository / branch, working directory, context usage
// and the provider's usage limits (share used, e.g. 5h session and weekly window).
import type {
  ProviderInstanceId,
  ServerProvider,
  UsageLimitSourceSnapshots,
} from "@t3tools/contracts";
import type { ContextWindowSnapshot } from "@t3tools/client-runtime/state/context-window";
import { collectProviderUsageLimits, remainingPercent } from "@t3tools/shared/usageLimits";
import { useNavigate } from "@tanstack/react-router";
import type { ReactNode } from "react";
import { FolderIcon, GaugeIcon, GitBranchIcon } from "lucide-react";

import { openCommandPalette } from "../../commandPaletteBus";

import { useNowMinute } from "../../hooks/useNowMinute";
import { formatContextWindowTokens } from "../../lib/contextWindow";
import { cn } from "../../lib/utils";

const STRIP_BUTTON =
  "inline-flex h-6 cursor-pointer items-center gap-1 rounded-md border border-transparent px-1.75 transition-colors duration-150 hover:text-foreground";

function usageLabel(label: string): string {
  return /^session$/i.test(label) ? "5h" : label;
}

function toneOf(used: number): string {
  return used >= 90 ? "text-destructive" : used >= 80 ? "text-warning" : "";
}

function shortenPath(path: string): string {
  return path.replace(/^\/home\/[^/]+/, "~");
}

export function ComposerStatusStrip(props: {
  readonly className?: string;
  readonly trailing?: ReactNode;
  readonly contextWindow: ContextWindowSnapshot | null;
  readonly providerInstanceId: ProviderInstanceId | null;
  readonly providers: readonly ServerProvider[];
  readonly sources: UsageLimitSourceSnapshots;
  readonly branch: string | null;
  readonly repository: string | null;
  readonly directory: string | null;
  readonly onPickDirectory: () => void;
}) {
  const navigate = useNavigate();
  const nowMinute = useNowMinute();
  const now = Date.parse(`${nowMinute}:00Z`);
  const report =
    props.providerInstanceId !== null
      ? collectProviderUsageLimits(props.providerInstanceId, props.providers, props.sources, now)
      : null;
  const limits =
    report?.accounts.find((entry) => entry.instanceId === props.providerInstanceId)?.limits ??
    props.providers.find((provider) => provider.instanceId === props.providerInstanceId)
      ?.usageLimits;
  const windows = limits?.windows ?? [];

  const context = props.contextWindow;
  const maxTokens = context?.maxTokens != null && context.maxTokens > 0 ? context.maxTokens : null;
  const contextUsed =
    context && maxTokens !== null && context.usedPercentage !== null
      ? Math.round(Math.max(0, Math.min(100, context.usedPercentage)))
      : null;

  return (
    <div
      data-composer-status-strip="true"
      className={cn(
        "flex min-w-0 flex-wrap items-center gap-x-1 px-1.25 text-muted-foreground text-xs tabular-nums [&_svg]:size-3 [&_svg]:shrink-0",
        props.className,
      )}
    >
      <button
        type="button"
        title="Add a project or create a GitHub repository"
        onClick={() => openCommandPalette({ open: "add-project" })}
        className={cn(STRIP_BUTTON, "min-w-0")}
      >
        <GitBranchIcon aria-hidden />
        <span className="truncate">
          {props.repository
            ? `${props.repository}${props.branch ? ` / ${props.branch}` : ""}`
            : "No repo"}
        </span>
      </button>
      {props.directory ? (
        <button
          type="button"
          title="Choose a folder (starts a new chat there)"
          onClick={props.onPickDirectory}
          className={cn(STRIP_BUTTON, "min-w-0")}
        >
          <FolderIcon aria-hidden />
          <span className="truncate">{shortenPath(props.directory)}</span>
        </button>
      ) : null}
      {context ? (
        <span
          title={
            maxTokens !== null
              ? `Context: ${formatContextWindowTokens(context.usedTokens)} / ${formatContextWindowTokens(maxTokens)} tokens`
              : "Context"
          }
          className={cn(
            "inline-flex h-6 items-center gap-1 border border-transparent px-1.75",
            toneOf(contextUsed ?? 0),
          )}
        >
          <GaugeIcon aria-hidden />
          Context{" "}
          {contextUsed !== null ? `${contextUsed}%` : formatContextWindowTokens(context.usedTokens)}
        </span>
      ) : null}
      {windows.length > 0 ? (
        <button
          type="button"
          title="Open usage"
          onClick={() => void navigate({ to: "/usage" })}
          className={cn(STRIP_BUTTON, "gap-3")}
        >
          {windows.map((window) => {
            const used = 100 - remainingPercent(window);
            return (
              <span key={window.id} className={cn("whitespace-nowrap", toneOf(used))}>
                {usageLabel(window.label)} {used}%
              </span>
            );
          })}
        </button>
      ) : null}
      {props.trailing ? (
        <span className={cn("inline-flex", windows.length === 0 && "ms-auto")}>
          {props.trailing}
        </span>
      ) : null}
    </div>
  );
}
