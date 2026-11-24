// CC3: shows the completion overlay when a Claude chat stops working while CC3 is in the background,
// and opens the chat when the overlay is clicked.
import { useNavigate, useParams } from "@tanstack/react-router";
import { useGT } from "gt-react";
import { useEffect, useRef, useState } from "react";

import { claudeTuiStatusFromTitle, useClaudeTuiUnseen } from "../claudeTuiStatus";
import { useClaudeTuiTitles } from "../state/terminalSessions";
import { usePrimaryEnvironmentId } from "../state/environments";
import { buildThreadRouteParams } from "../threadRoutes";
import type { EnvironmentId, ThreadId } from "@t3tools/contracts";
import { scopeThreadRef } from "@t3tools/client-runtime/environment";

const chatName = (title: string) => title.replace(/^[^\p{L}\p{N}]+/u, "").trim();

export function Cc3CompletionNotifier() {
  const gt = useGT();
  const navigate = useNavigate();
  const environmentId = usePrimaryEnvironmentId();
  const titles = useClaudeTuiTitles(environmentId);
  const previous = useRef(new Map<string, string | null>());
  const activeThreadId = useParams({ strict: false, select: (params) => params.threadId });
  const markUnseen = useClaudeTuiUnseen((state) => state.mark);
  const clearUnseen = useClaudeTuiUnseen((state) => state.clear);
  const [focused, setFocused] = useState(() => document.hasFocus());

  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- window focus is an external signal
  useEffect(() => {
    const update = () => setFocused(document.hasFocus());
    window.addEventListener("focus", update);
    window.addEventListener("blur", update);
    return () => {
      window.removeEventListener("focus", update);
      window.removeEventListener("blur", update);
    };
  }, []);

  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- viewing a chat in the focused window clears its dot
  useEffect(() => {
    if (focused && activeThreadId) clearUnseen(activeThreadId);
  }, [activeThreadId, clearUnseen, focused]);

  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- reacts to terminal title changes pushed by the server and talks to the desktop shell
  useEffect(() => {
    if (environmentId === null) return;
    for (const { threadId, title } of titles) {
      const status = claudeTuiStatusFromTitle(title);
      const before = previous.current.get(threadId);
      previous.current.set(threadId, status);
      if (status === "working") clearUnseen(threadId);
      const viewing = document.hasFocus() && threadId === activeThreadId;
      if (before === "working" && status === "done" && !viewing) markUnseen(threadId);
      if (before !== "working" || (status !== "done" && status !== "input")) continue;
      if (document.hasFocus()) continue;
      void window.desktopBridge?.showCompletionOverlay({
        title: chatName(title) || gt("Claude"),
        message: status === "input" ? gt("Needs your input") : gt("Finished"),
        environmentId,
        threadId,
      });
    }
  }, [activeThreadId, clearUnseen, environmentId, gt, markUnseen, titles]);

  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- the desktop shell reports a click on the overlay as a window event
  useEffect(() => {
    const open = (event: Event) => {
      const detail = (event as CustomEvent<{ environmentId: string; threadId: string }>).detail;
      void navigate({
        to: "/$environmentId/$threadId",
        params: buildThreadRouteParams(
          scopeThreadRef(detail.environmentId as EnvironmentId, detail.threadId as ThreadId),
        ),
      });
    };
    window.addEventListener("cc3-open-thread", open);
    return () => window.removeEventListener("cc3-open-thread", open);
  }, [navigate]);

  return null;
}
