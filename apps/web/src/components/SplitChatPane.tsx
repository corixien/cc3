// CC3: the second chat of the split view. Shows any chat next to the routed one; its header can
// swap the two, drop the left chat (this one becomes the only chat) or close the split.
import type { ScopedThreadRef } from "@t3tools/contracts";
import { useNavigate } from "@tanstack/react-router";
import { useGT } from "gt-react";
import { ArrowLeftRightIcon } from "lucide-react";
import { useEffect } from "react";

import { useQuietThreadShell } from "../state/entities";
import { useSplitViewStore } from "../splitViewStore";
import { buildThreadRouteParams } from "../threadRoutes";
import ChatView from "./ChatView";
import { Button } from "./ui/button";
import { Tooltip, TooltipPopup, TooltipTrigger } from "./ui/tooltip";

function sameThread(a: ScopedThreadRef | null, b: ScopedThreadRef | null) {
  return (
    a !== null && b !== null && a.environmentId === b.environmentId && a.threadId === b.threadId
  );
}

export function SplitChatPane({ routeThread }: { routeThread: ScopedThreadRef | null }) {
  const gt = useGT();
  const navigate = useNavigate();
  const secondary = useSplitViewStore((state) => state.secondary);
  const open = useSplitViewStore((state) => state.open);
  const focus = useSplitViewStore((state) => state.focus);
  const ratio = useSplitViewStore((state) => state.ratio);
  const setFocus = useSplitViewStore((state) => state.setFocus);
  const setRatio = useSplitViewStore((state) => state.setRatio);
  const shell = useQuietThreadShell(secondary);
  // Opening the split chat on the left makes the split pointless.
  const active = secondary !== null && routeThread !== null && !sameThread(secondary, routeThread);

  // The split is styled from the document root: both panes share one flex row whose left pane
  // belongs to the router, so the ratio, the dimming and the focus are CSS variables there.
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- mirrors split state onto the document root for global CSS
  useEffect(() => {
    const root = document.documentElement;
    if (!active) {
      delete root.dataset.splitActive;
      delete root.dataset.splitFocus;
      return;
    }
    root.dataset.splitActive = "";
    root.dataset.splitFocus = focus;
    root.style.setProperty("--cc3-split-left", String(ratio));
    root.style.setProperty("--cc3-split-right", String(1 - ratio));
    return () => {
      delete root.dataset.splitActive;
      delete root.dataset.splitFocus;
    };
  }, [active, focus, ratio]);

  // The window controls at the top right belong to the left chat; keep them out of the right pane.
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- mirrors the pane width onto the document root for global CSS
  useEffect(() => {
    if (!active) return;
    const pane = document.querySelector<HTMLElement>("[data-split-chat-pane]");
    if (!pane) return;
    const root = document.documentElement;
    const observer = new ResizeObserver(() =>
      root.style.setProperty("--cc3-split-right-width", `${pane.getBoundingClientRect().width}px`),
    );
    observer.observe(pane);
    return () => {
      observer.disconnect();
      root.style.removeProperty("--cc3-split-right-width");
    };
  }, [active]);

  // Pressing inside a chat focuses its pane; the sidebar leaves the focus alone.
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- document-level pointer listener
  useEffect(() => {
    if (!active) return;
    const onPointerDown = (event: PointerEvent) => {
      const target = event.target instanceof Element ? event.target : null;
      if (target?.closest("[data-split-chat-pane]")) setFocus("right");
      else if (target?.closest('main[data-slot="sidebar-inset"]')) setFocus("left");
    };
    document.addEventListener("pointerdown", onPointerDown, true);
    return () => document.removeEventListener("pointerdown", onPointerDown, true);
  }, [active, setFocus]);

  if (!active || secondary === null || routeThread === null) return null;

  const openOnLeft = (ref: ScopedThreadRef) =>
    void navigate({ to: "/$environmentId/$threadId", params: buildThreadRouteParams(ref) });
  const swap = () => {
    open(routeThread);
    openOnLeft(secondary);
  };
  // Dragging the divider changes the share of the two panes.
  const startResize = (event: React.PointerEvent<HTMLDivElement>) => {
    event.preventDefault();
    const pane = event.currentTarget.parentElement;
    const left = document.querySelector<HTMLElement>('main[data-slot="sidebar-inset"]');
    if (!pane || !left) return;
    const handle = event.currentTarget;
    handle.setPointerCapture(event.pointerId);
    const move = (moveEvent: PointerEvent) => {
      const start = left.getBoundingClientRect().left;
      const end = pane.getBoundingClientRect().right;
      setRatio((moveEvent.clientX - start) / (end - start));
    };
    const stop = () => {
      handle.removeEventListener("pointermove", move);
      handle.removeEventListener("pointerup", stop);
      handle.removeEventListener("pointercancel", stop);
    };
    handle.addEventListener("pointermove", move);
    handle.addEventListener("pointerup", stop);
    handle.addEventListener("pointercancel", stop);
  };

  return (
    <aside
      data-split-chat-pane
      className="relative flex h-dvh min-h-0 min-w-0 flex-col border-l border-border bg-background"
      style={{ flex: `${1 - ratio} 1 0%` }}
    >
      <div
        role="separator"
        aria-orientation="vertical"
        aria-label={gt("Resize split view")}
        data-split-divider
        onPointerDown={startResize}
        onDoubleClick={() => setRatio(0.5)}
        className="group/divider absolute inset-y-0 -left-1.5 z-30 flex w-3 cursor-col-resize touch-none items-center justify-center"
      >
        <div className="h-full w-px bg-transparent transition-colors group-hover/divider:bg-primary/60" />
        <Tooltip>
          <TooltipTrigger
            render={
              <Button
                variant="outline"
                size="icon-xs"
                aria-label={gt("Swap sides")}
                onPointerDown={(event) => event.stopPropagation()}
                onClick={swap}
                className="absolute top-1/2 -translate-y-1/2 rounded-full bg-background opacity-0 shadow-sm transition-opacity group-hover/divider:opacity-100 focus-visible:opacity-100"
              />
            }
          >
            <ArrowLeftRightIcon />
          </TooltipTrigger>
          <TooltipPopup side="bottom">{gt("Swap sides")}</TooltipPopup>
        </Tooltip>
      </div>
      {shell === null ? (
        <p className="p-3 text-xs text-muted-foreground">{gt("This chat is not available.")}</p>
      ) : (
        <ChatView
          key={`${secondary.environmentId}:${secondary.threadId}`}
          environmentId={secondary.environmentId}
          threadId={secondary.threadId}
          routeKind="server"
          splitPane
        />
      )}
    </aside>
  );
}
