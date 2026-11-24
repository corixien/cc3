// CC3 experiment: the chat area of a Claude thread is the real Claude Code TUI in a terminal.
// The thread still exists on the server, so the sidebar, project, files and browser work as before.
import {
  type EnvironmentId,
  type ModelSelection,
  type ProjectId,
  type ResolvedKeybindingsConfig,
  type RuntimeMode,
  type ThreadId,
} from "@t3tools/contracts";
import { scopeThreadRef } from "@t3tools/client-runtime/environment";
import { useGT } from "gt-react";
import { useEffect, useRef, useState } from "react";

import {
  CLAUDE_TUI_TERMINAL_ID,
  DEFAULT_INTERACTION_MODE,
  DEFAULT_THREAD_TERMINAL_ID,
} from "../types";
import { revealThreadTerminal } from "../lib/revealThreadTerminal";
import { terminalEnvironment } from "../state/terminal";
import { useAtomCommand } from "../state/use-atom-command";
import { finalizePromotedDraftThreadByRef } from "../composerDraftStore";
import { threadEnvironment } from "../state/threads";
import { useOrchestrationCommand } from "../state/use-orchestration-command";
import { claudeTuiStatusFromTitle, setClaudeTuiStatus } from "../claudeTuiStatus";
import { useStableCallback } from "../hooks/useStableCallback";
import { useClientSettings } from "../hooks/useSettings";
import { Button } from "./ui/button";
import { TerminalViewport } from "./ThreadTerminalDrawer";

export interface ClaudeTuiPaneProps {
  readonly environmentId: EnvironmentId;
  readonly threadId: ThreadId;
  readonly projectId: ProjectId;
  readonly isDraft: boolean;
  readonly cwd: string;
  readonly worktreePath: string | null;
  readonly branch: string | null;
  readonly modelSelection: ModelSelection;
  readonly runtimeMode: RuntimeMode;
  readonly keybindings: ResolvedKeybindingsConfig;
  readonly onNewChat: () => void;
}

const THEME_VARIABLES = {
  "--terminal-background": "#000000",
  "--terminal-foreground": "#e8e8e8",
  "--terminal-cursor": "#e8e8e8",
} as React.CSSProperties;

export function ClaudeTuiPane(props: ClaudeTuiPaneProps) {
  const gt = useGT();
  const { environmentId, threadId, isDraft } = props;
  const createThread = useOrchestrationCommand(threadEnvironment.create, {
    reportFailure: true,
  });
  const skipPermissions = useClientSettings((settings) => settings.claudeSkipPermissions);
  const updateMetadata = useOrchestrationCommand(threadEnvironment.updateMetadata, {
    reportFailure: false,
  });
  const lastTitleRef = useRef<string | null>(null);
  const [fontReady, setFontReady] = useState(false);
  const [threadReady, setThreadReady] = useState(!isDraft);
  const [exited, setExited] = useState(false);
  const [restarts, setRestarts] = useState(0);
  const createdRef = useRef(false);

  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- the terminal measures its cells, so the font must be loaded first.
  useEffect(() => {
    void Promise.all([
      document.fonts.load('16px "CC3 Mono"'),
      document.fonts.load('bold 16px "CC3 Mono"'),
    ])
      .catch(() => undefined)
      .then(() => setFontReady(true));
  }, []);

  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- a draft needs its server thread before the TUI starts, so the chat shows up in the sidebar.
  useEffect(() => {
    if (!isDraft || createdRef.current) return;
    createdRef.current = true;
    void createThread({
      environmentId,
      input: {
        threadId,
        projectId: props.projectId,
        title: gt("Claude"),
        modelSelection: props.modelSelection,
        runtimeMode: props.runtimeMode,
        interactionMode: DEFAULT_INTERACTION_MODE,
        branch: props.branch,
        worktreePath: props.worktreePath,
      },
    }).then((result) => {
      if (result._tag === "Success") {
        setThreadReady(true);
        finalizePromotedDraftThreadByRef(scopeThreadRef(environmentId, threadId));
      } else {
        createdRef.current = false;
      }
    });
  });

  // Claude sets the terminal title to its session summary; the chat list shows it.
  const handleTitle = useStableCallback((raw: string) => {
    const status = claudeTuiStatusFromTitle(raw);
    if (status !== null) setClaudeTuiStatus(threadId, status);
    const title = raw
      .replace(/^[^\p{L}\p{N}]+/u, "")
      .trim()
      .slice(0, 80);
    if (!threadReady || title.length === 0 || /^claude( code)?$/i.test(title)) return;
    if (title === lastTitleRef.current) return;
    lastTitleRef.current = title;
    void updateMetadata({ environmentId, input: { threadId, title } });
  });
  // `cc3-terminal` (run by Claude) types a command into the thread's terminal drawer for the user.
  const writeTerminal = useAtomCommand(terminalEnvironment.write, { reportFailure: false });
  const handleTerminalCommand = useStableCallback((command: string) => {
    revealThreadTerminal(scopeThreadRef(environmentId, threadId), DEFAULT_THREAD_TERMINAL_ID);
    // The drawer's shell may still be starting; retry the write briefly.
    const attempt = (remaining: number) => {
      void writeTerminal({
        environmentId,
        input: { threadId, terminalId: DEFAULT_THREAD_TERMINAL_ID, data: command },
      }).then((result) => {
        if (result._tag === "Failure" && remaining > 0) {
          window.setTimeout(() => attempt(remaining - 1), 400);
        }
      });
    };
    window.setTimeout(() => attempt(6), 300);
  });
  const handleSubmitLine = useStableCallback((line: string) => {
    if (line !== "/new") return false;
    props.onNewChat();
    return true;
  });

  return (
    <div
      className="relative min-h-0 flex-1 bg-black p-2 pl-3"
      style={THEME_VARIABLES}
      data-claude-tui-pane
    >
      {fontReady && threadReady ? (
        <TerminalViewport
          key={restarts}
          advancedTypography
          threadRef={scopeThreadRef(environmentId, threadId)}
          threadId={threadId}
          terminalId={CLAUDE_TUI_TERMINAL_ID}
          terminalLabel="Claude"
          cwd={props.cwd}
          worktreePath={props.worktreePath}
          runtimeEnv={{
            CC3_TUI_SESSION: threadId,
            ...(skipPermissions ? { CC3_TUI_SKIP_PERMISSIONS: "1" } : {}),
          }}
          providerInstanceId={props.modelSelection.instanceId}
          onSessionExited={() => setExited(true)}
          onTitleChange={handleTitle}
          onTerminalCommand={handleTerminalCommand}
          onSubmitLine={handleSubmitLine}
          autoFocus
          visible
          keybindings={props.keybindings}
        />
      ) : null}
      {exited ? (
        <div className="absolute inset-x-0 bottom-3 flex justify-center">
          <Button
            size="sm"
            onClick={() => {
              setExited(false);
              setRestarts((count) => count + 1);
            }}
          >
            {gt("Restart Claude")}
          </Button>
        </div>
      ) : null}
    </div>
  );
}
