// CC3: split view. The routed chat stays on the left; `secondary` is the chat shown beside it.
// `focus` is the pane that sidebar clicks replace; `ratio` is the left pane's share of the width.
import type { ScopedThreadRef } from "@t3tools/contracts";
import { create } from "zustand";

export type SplitFocus = "left" | "right";

export const SPLIT_RATIO_MIN = 0.25;
export const SPLIT_RATIO_MAX = 0.75;

interface SplitViewState {
  readonly secondary: ScopedThreadRef | null;
  readonly focus: SplitFocus;
  readonly ratio: number;
  readonly open: (ref: ScopedThreadRef) => void;
  readonly close: () => void;
  readonly setFocus: (focus: SplitFocus) => void;
  readonly setRatio: (ratio: number) => void;
}

export const useSplitViewStore = create<SplitViewState>()((set) => ({
  secondary: null,
  focus: "right",
  ratio: 0.5,
  open: (ref) => set({ secondary: ref, focus: "right" }),
  close: () => set({ secondary: null, focus: "left" }),
  setFocus: (focus) => set((state) => (state.focus === focus ? state : { focus })),
  setRatio: (ratio) => set({ ratio: Math.min(SPLIT_RATIO_MAX, Math.max(SPLIT_RATIO_MIN, ratio)) }),
}));
