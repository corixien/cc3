// CC3: one compact line under the composer with context usage, git branch, repository, working
// directory and the provider's usage limits (share used, e.g. 5h session and weekly window).
import type {
  ProviderInstanceId,
  ServerProvider,
  UsageLimitSourceSnapshots,
} from "@t3tools/contracts";
import type { ContextWindowSnapshot } from "@t3tools/client-runtime/state/context-window";
import { collectProviderUsageLimits, remainingPercent } from "@t3tools/shared/usageLimits";
import { useNavigate } from "@tanstack/react-router";
import { FolderIcon, GaugeIcon, GitBranchIcon } from "lucide-react";

import { openCommandPalette } from "../../commandPaletteBus";

import { useNowMinute } from "../../hooks/useNowMinute";
import { formatContextWindowTokens } from "../../lib/contextWindow";
import { cn } from "../../lib/utils";

function toneOf(used: number): string {
  return used >= 90 ? "text-destructive" : used >= 80 ? "text-warning" : "";
}

function shortenPath(path: string): string {
  return path.replace(/^\/home\/[^/]+/, "~");
}

export function ComposerStatusStrip(props: {
  readonly contextWindow: ContextWindowSnapshot | null;
  readonly providerInstanceId: ProviderInstanceId | null;
  readonly providers: readonly ServerProvider[];
  readonly sources: UsageLimitSourceSnapshots;
  readonly branch: string | null;
  readonly repository: string | null;
  readonly directory: string | null;
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
      className="mx-auto flex w-full min-w-0 max-w-(--chat-content-max-width) flex-wrap items-center gap-x-3 gap-y-0.5 px-3 pt-1 pb-1.5 text-muted-foreground text-xs tabular-nums [&_svg]:size-3.5 [&_svg]:shrink-0"
    >
      {context ? (
        <span className={cn("inline-flex items-center gap-1", toneOf(contextUsed ?? 0))}>
          <GaugeIcon aria-hidden />
          {maxTokens !== null
            ? `${formatContextWindowTokens(context.usedTokens)} / ${formatContextWindowTokens(maxTokens)}${contextUsed !== null ? ` (${contextUsed}%)` : ""}`
            : formatContextWindowTokens(context.usedTokens)}
        </span>
      ) : null}
      {props.branch ? (
        <span className="inline-flex min-w-0 items-center gap-1">
          <GitBranchIcon aria-hidden />
          <span className="truncate">
            {props.repository ? `${props.repository} / ` : ""}
            {props.branch}
          </span>
        </span>
      ) : props.repository ? (
        <span className="truncate">{props.repository}</span>
      ) : null}
      {props.directory ? (
        <button
          type="button"
          title="Change directory"
          onClick={() => openCommandPalette({ open: "new-thread-in" })}
          className="inline-flex min-w-0 cursor-pointer items-center gap-1 hover:text-foreground"
        >
          <FolderIcon aria-hidden />
          <span className="truncate">{shortenPath(props.directory)}</span>
        </button>
      ) : null}
      {windows.length > 0 ? (
        <button
          type="button"
          title="Open usage"
          onClick={() => void navigate({ to: "/usage" })}
          className="ms-auto inline-flex cursor-pointer items-center gap-3 hover:text-foreground"
        >
          {windows.map((window) => {
            const used = 100 - remainingPercent(window);
            return (
              <span key={window.id} className={cn("whitespace-nowrap", toneOf(used))}>
                {window.label} {used}%
              </span>
            );
          })}
        </button>
      ) : null}
    </div>
  );
}
