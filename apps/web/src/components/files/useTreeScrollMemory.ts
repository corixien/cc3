// CC3: the file tree unmounts while a file is open; remembers its scroll position per root so closing the file returns to the same spot.
import { type RefObject, useEffect } from "react";

const scrollMemory = new Map<string, number>();

const scrollerOf = (root: HTMLElement) =>
  root
    .querySelector("file-tree-container")
    ?.shadowRoot?.querySelector<HTMLElement>("[data-file-tree-virtualized-scroll]") ?? null;

export function useTreeScrollMemory(rootRef: RefObject<HTMLElement | null>, key: string) {
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- tracks and restores the scroll offset of a shadow-DOM scroller
  useEffect(() => {
    const root = rootRef.current;
    if (!root) return;
    const target = scrollMemory.get(key) ?? 0;
    let scroller: HTMLElement | null = null;
    let restored = target === 0;
    let frame = 0;
    const startedAt = performance.now();
    const onScroll = () => {
      if (restored && scroller) scrollMemory.set(key, scroller.scrollTop);
    };
    const tick = () => {
      const found = scrollerOf(root);
      if (found && found !== scroller) {
        scroller?.removeEventListener("scroll", onScroll);
        scroller = found;
        scroller.addEventListener("scroll", onScroll, { passive: true });
      }
      if (!restored && scroller) {
        // Folders reopen as they load, so wait until the list is tall enough.
        if (scroller.scrollHeight - scroller.clientHeight >= target) {
          scroller.scrollTop = target;
          restored = true;
        } else if (performance.now() - startedAt > 3000) restored = true;
      }
      if (!scroller || !restored) frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => {
      cancelAnimationFrame(frame);
      scroller?.removeEventListener("scroll", onScroll);
    };
  }, [rootRef, key]);
}
