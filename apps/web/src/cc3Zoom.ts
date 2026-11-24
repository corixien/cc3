// CC3: Ctrl +/-/0 outside a terminal zooms the window (Electron zoom, so layout and the browser panel stay
// correct). Terminals handle the same keys themselves and cancel the window zoom, so each zooms on its own.
import { useSyncExternalStore } from "react";

const STORAGE_KEY = "cc3:gui-zoom";
const STEP = 0.1;
const MIN = 0.5;
const MAX = 2;

let zoom = 1;
const listeners = new Set<() => void>();

function clamp(value: number): number {
  return Math.min(MAX, Math.max(MIN, Math.round(value * 10) / 10));
}

function setZoom(next: number): void {
  zoom = clamp(next);
  window.desktopBridge?.setWindowZoomFactor(zoom);
  localStorage.setItem(STORAGE_KEY, String(zoom));
  for (const listener of listeners) listener();
}

export const getGuiZoom = () => zoom;

export function useGuiZoom(): number {
  return useSyncExternalStore(
    (listener) => {
      listeners.add(listener);
      return () => listeners.delete(listener);
    },
    () => zoom,
    () => 1,
  );
}

export function installGuiZoom(): void {
  if (!window.desktopBridge) return;
  const stored = Number(localStorage.getItem(STORAGE_KEY));
  setZoom(Number.isFinite(stored) && stored > 0 ? stored : 1);
  window.addEventListener("keydown", (event) => {
    if (event.defaultPrevented || !event.ctrlKey || event.altKey || event.metaKey) return;
    const next =
      event.key === "+" || event.key === "="
        ? zoom + STEP
        : event.key === "-" || event.key === "_"
          ? zoom - STEP
          : event.key === "0"
            ? 1
            : null;
    if (next === null) return;
    event.preventDefault();
    setZoom(next);
  });
}
