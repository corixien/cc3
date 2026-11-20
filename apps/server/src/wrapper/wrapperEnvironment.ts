// CC3 wrapper: environment handling for CLIs run as interactive TUIs under a PTY.

const INHERITED_CLAUDE_MARKERS =
  /^(CLAUDE_CODE_|CLAUDECODE$|CLAUDE_AGENT_|CLAUDE_PREVIEW_|CLAUDE_DESKTOP_)/;

/**
 * A CLI started from inside another Claude session inherits markers that switch off its
 * transcript (`CLAUDE_CODE_CHILD_SESSION`) or change its entrypoint. The wrapper reads the
 * transcript, so the markers are dropped; user-facing settings such as `CLAUDE_CONFIG_DIR` stay.
 */
export function scrubWrapperEnvironment(environment: NodeJS.ProcessEnv): NodeJS.ProcessEnv {
  const scrubbed: NodeJS.ProcessEnv = {};
  for (const [key, value] of Object.entries(environment)) {
    if (value === undefined || INHERITED_CLAUDE_MARKERS.test(key)) continue;
    scrubbed[key] = value;
  }
  scrubbed.TERM = scrubbed.TERM ?? "xterm-256color";
  return scrubbed;
}
