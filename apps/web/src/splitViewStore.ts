// CC3: split view. The routed chat stays on the left; `secondary` is the chat shown beside it.
import type { ScopedThreadRef } from "@t3tools/contracts";
import { create } from "zustand";

interface SplitViewState {
  readonly secondary: ScopedThreadRef | null;
  readonly open: (ref: ScopedThreadRef) => void;
  readonly close: () => void;
}

export const useSplitViewStore = create<SplitViewState>()((set) => ({
  secondary: null,
  open: (ref) => set({ secondary: ref }),
  close: () => set({ secondary: null }),
}));
