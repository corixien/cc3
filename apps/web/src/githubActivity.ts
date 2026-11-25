// CC3: a counter that moves when something may have changed in git or on GitHub (a git or gh command of
// Claude, or CC3 regaining focus), so the GitHub panel reloads. Bursts collapse into one update.
import { useSyncExternalStore } from "react";

let count = 0;
let timer: ReturnType<typeof setTimeout> | undefined;
const listeners = new Set<() => void>();

export function bumpGitActivity() {
  clearTimeout(timer);
  timer = setTimeout(() => {
    count += 1;
    for (const listener of listeners) listener();
  }, 800);
}

function subscribe(listener: () => void) {
  listeners.add(listener);
  window.addEventListener("focus", bumpGitActivity);
  return () => {
    listeners.delete(listener);
    window.removeEventListener("focus", bumpGitActivity);
  };
}

export const useGitActivity = () =>
  useSyncExternalStore(
    subscribe,
    () => count,
    () => 0,
  );
