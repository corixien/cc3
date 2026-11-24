// CC3: typed access to the desktop host's GitHub operations (see apps/desktop/src/cc3GitHub.ts).
export type GhResult = { ok: true; data: unknown } | { ok: false; error: string };

export const callGh = (request: Record<string, unknown>): Promise<GhResult> =>
  window.desktopBridge?.github(request) ??
  Promise.resolve({ ok: false, error: "GitHub needs the CC3 desktop app." });
