// @effect-diagnostics globalTimers:off - idle timers for standby sessions
// CC3 wrapper: the Claude TUIs the server keeps for a chat. A chat with a running turn loop has a
// live one (owned by the query runner); an open chat without one gets a standby session so CLI
// commands work before its first message. The standby is dropped once the live one registers.

export interface LiveTuiHandle {
  /** Runs a slash command and returns what it printed. */
  readonly runCommand: (command: string) => Promise<string>;
  /** True while a model turn is running; commands then wait for the turn instead of typing over it. */
  readonly isBusy: () => boolean;
  readonly close: () => void;
}

const STANDBY_IDLE_MS = 10 * 60_000;

const live = new Map<string, LiveTuiHandle>();
const standby = new Map<string, { handle: LiveTuiHandle; timer: ReturnType<typeof setTimeout> }>();

export function registerLiveTui(threadId: string, handle: LiveTuiHandle): void {
  closeStandbyTui(threadId);
  live.set(threadId, handle);
}

export function unregisterLiveTui(threadId: string, handle: LiveTuiHandle): void {
  if (live.get(threadId) === handle) live.delete(threadId);
}

export function findLiveTui(threadId: string): LiveTuiHandle | undefined {
  return live.get(threadId);
}

export function findStandbyTui(threadId: string): LiveTuiHandle | undefined {
  const entry = standby.get(threadId);
  if (entry === undefined) return undefined;
  clearTimeout(entry.timer);
  entry.timer = setTimeout(() => closeStandbyTui(threadId), STANDBY_IDLE_MS);
  return entry.handle;
}

export function setStandbyTui(threadId: string, handle: LiveTuiHandle): void {
  closeStandbyTui(threadId);
  standby.set(threadId, {
    handle,
    timer: setTimeout(() => closeStandbyTui(threadId), STANDBY_IDLE_MS),
  });
}

export function closeStandbyTui(threadId: string): void {
  const entry = standby.get(threadId);
  if (entry === undefined) return;
  clearTimeout(entry.timer);
  standby.delete(threadId);
  entry.handle.close();
}
