// CC3: the second chat of the split view. Shows any chat next to the routed one; its header can
// swap the two, drop the left chat (this one becomes the only chat) or close the split.
import type { ScopedThreadRef } from "@t3tools/contracts";
import { useNavigate } from "@tanstack/react-router";
import { useGT } from "gt-react";
import { ArrowLeftRightIcon, PanelLeftCloseIcon, XIcon } from "lucide-react";

import { ComposerHandleContext } from "../composerHandleContext";
import { useQuietThreadShell } from "../state/entities";
import { useSplitViewStore } from "../splitViewStore";
import { buildThreadRouteParams } from "../threadRoutes";
import ChatView from "./ChatView";
import { BackgroundThreadContext } from "./chat/backgroundThread";
import { Button } from "./ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "./ui/tooltip";

const NO_PARENT_PANEL = { openToolDiff: () => {}, runShellCommand: () => {} };

function sameThread(a: ScopedThreadRef | null, b: ScopedThreadRef | null) {
  return (
    a !== null && b !== null && a.environmentId === b.environmentId && a.threadId === b.threadId
  );
}

export function SplitChatPane({ routeThread }: { routeThread: ScopedThreadRef | null }) {
  const gt = useGT();
  const navigate = useNavigate();
  const secondary = useSplitViewStore((state) => state.secondary);
  const close = useSplitViewStore((state) => state.close);
  const open = useSplitViewStore((state) => state.open);
  const shell = useQuietThreadShell(secondary);
  // Opening the split chat on the left makes the split pointless.
  if (secondary === null || routeThread === null || sameThread(secondary, routeThread)) return null;

  const openOnLeft = (ref: ScopedThreadRef) =>
    void navigate({ to: "/$environmentId/$threadId", params: buildThreadRouteParams(ref) });
  const button = (label: string, onClick: () => void, icon: React.ReactNode) => (
    <Tooltip>
      <TooltipTrigger
        render={<Button variant="ghost" size="icon-xs" aria-label={label} onClick={onClick} />}
      >
        {icon}
      </TooltipTrigger>
      <TooltipPopup side="bottom">{label}</TooltipPopup>
    </Tooltip>
  );

  return (
    <aside
      data-split-chat-pane
      className="flex h-dvh min-h-0 min-w-0 flex-1 basis-0 flex-col border-l border-border bg-background"
    >
      <div className="drag-region flex h-[var(--workspace-topbar-height)] min-h-[var(--workspace-topbar-height)] shrink-0 items-center gap-1 pl-3 pr-2 wco:pr-(--workspace-native-controls-inset)">
        <span className="min-w-0 flex-1 truncate text-sm font-medium">
          {shell?.title ?? gt("Chat")}
        </span>
        {button(
          gt("Swap sides"),
          () => {
            open(routeThread);
            openOnLeft(secondary);
          },
          <ArrowLeftRightIcon />,
        )}
        {button(
          gt("Remove the left chat (this chat takes the full width)"),
          () => {
            close();
            openOnLeft(secondary);
          },
          <PanelLeftCloseIcon />,
        )}
        {button(gt("Close split view"), close, <XIcon />)}
      </div>
      {shell === null ? (
        <p className="p-3 text-xs text-muted-foreground">{gt("This chat is not available.")}</p>
      ) : (
        <BackgroundThreadContext value>
          <ComposerHandleContext value={null}>
            <ChatView
              key={`${secondary.environmentId}:${secondary.threadId}`}
              environmentId={secondary.environmentId}
              threadId={secondary.threadId}
              routeKind="server"
              embedded={NO_PARENT_PANEL}
            />
          </ComposerHandleContext>
        </BackgroundThreadContext>
      )}
    </aside>
  );
}
