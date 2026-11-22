// CC3 split view: both panes mount a full ChatView, but window-level shortcuts and menu actions
// belong to the pane that has the focus.
import { useSplitViewStore, type SplitFocus } from "../splitViewStore";

export function splitPaneOwnsTarget(pane: SplitFocus, target: EventTarget | null): boolean {
  const { secondary, focus } = useSplitViewStore.getState();
  if (secondary === null) return pane === "left";
  if (target instanceof Element) {
    if (target.closest("[data-split-chat-pane]")) return pane === "right";
    if (target.closest('main[data-slot="sidebar-inset"]')) return pane === "left";
  }
  return pane === focus;
}
