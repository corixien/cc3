import { useGT } from "gt-react";

import { useClaudeTuiStatus } from "../../claudeTuiStatus";
import { cn } from "../../lib/utils";

export function ClaudeTuiStatusDot({ threadId }: { threadId: string }) {
  const gt = useGT();
  const status = useClaudeTuiStatus(threadId);
  if (status === null) return null;
  const label =
    status === "working"
      ? gt("Claude is working")
      : status === "input"
        ? gt("Claude needs your input")
        : gt("Claude is done");
  return (
    <span
      role="img"
      aria-label={label}
      className={cn(
        "inline-block size-2 shrink-0 rounded-full",
        status === "working" &&
          "size-2.5 animate-spin rounded-full border-2 border-muted-foreground/30 border-t-foreground",
        status === "input" && "animate-pulse bg-warning",
        status === "done" && "bg-success",
      )}
    />
  );
}
