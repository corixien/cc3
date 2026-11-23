// CC3: popup above the composer for Claude Code commands such as /mcp or /plugins. It looks like the
// question panel and lists what the command is about so the user can pick right there.
import { useNavigate } from "@tanstack/react-router";
import { useGT } from "gt-react";
import { BlocksIcon, KeyRoundIcon, PlugIcon, ServerCogIcon } from "lucide-react";
import { useEffect } from "react";

import type { Cc3CommandPanelKind } from "../../cc3LocalSlashCommands";
import { cn } from "../../lib/utils";
import { Cc3McpManager, Cc3PluginsManager, type Cc3RunClaudeCommand } from "./Cc3ClaudeManager";
import { ComposerBanner } from "./ComposerBanner";
import {
  Row,
  ThreadDetailsOverview,
  type ThreadDetailsOverviewInput,
} from "./ThreadDetailsOverview";

export interface Cc3CommandPanelProps {
  readonly panel: Cc3CommandPanelKind;
  readonly overview: ThreadDetailsOverviewInput;
  readonly mcpServers: ReadonlyArray<{ readonly name: string; readonly status: string }>;
  readonly cli?: {
    readonly command: string;
    readonly output: string;
    readonly pending: boolean;
  } | null;
  /** Runs a Claude CLI command; set for Claude chats, which get the full plugin and MCP managers. */
  readonly runClaudeCommand?: Cc3RunClaudeCommand | undefined;
  readonly onClose: () => void;
}

export function Cc3CommandPanel(props: Cc3CommandPanelProps) {
  const gt = useGT();
  const navigate = useNavigate();
  const { onClose } = props;
  const open = (to: string) => {
    onClose();
    void navigate({ to: to as never });
  };

  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- Escape closes the popup, a document-level listener.
  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") onClose();
    };
    document.addEventListener("keydown", onKeyDown);
    return () => document.removeEventListener("keydown", onKeyDown);
  }, [onClose]);

  const servers = props.mcpServers;
  const title =
    props.panel === "cli"
      ? (props.cli?.command ?? "")
      : props.panel === "connectors"
        ? gt("Connectors")
        : props.panel === "overview"
          ? gt("Session")
          : gt("Plugins and integrations");

  return (
    <div className="mb-3">
      <ComposerBanner.Root detached data-cc3-command-panel="" variant="info">
        <div className="flex max-h-[min(24rem,50dvh)] flex-col gap-1 overflow-y-auto p-2">
          <div className="flex items-center justify-between px-2.5 pt-1 pb-1">
            <span className="text-sm font-semibold text-foreground">{title}</span>
            <ComposerBanner.Dismiss aria-label={gt("Close")} onClick={onClose} />
          </div>
          {props.panel === "cli" ? (
            props.cli?.pending ? (
              <p className="px-2.5 py-1 text-xs text-muted-foreground">{gt("Running...")}</p>
            ) : (
              <pre className="max-h-80 overflow-auto whitespace-pre-wrap px-2.5 py-1 font-mono text-xs text-foreground">
                {props.cli?.output}
              </pre>
            )
          ) : null}
          {props.panel === "overview" ? <ThreadDetailsOverview {...props.overview} /> : null}
          {props.panel === "connectors" && props.runClaudeCommand ? (
            <Cc3McpManager run={props.runClaudeCommand} />
          ) : null}
          {props.panel === "integrations" && props.runClaudeCommand ? (
            <Cc3PluginsManager run={props.runClaudeCommand} />
          ) : null}
          {props.panel === "connectors" && !props.runClaudeCommand ? (
            <>
              {servers.length === 0 ? (
                <p className="px-2.5 py-1 text-xs text-muted-foreground">
                  {gt("No MCP servers configured.")}
                </p>
              ) : (
                servers.map((server) => (
                  <Row
                    key={server.name}
                    icon={<PlugIcon />}
                    title={server.status}
                    onClick={() => open("/settings/integrations")}
                  >
                    <span className="min-w-0 truncate">{server.name}</span>
                    <span className="ms-auto shrink-0 text-xs font-normal text-muted-foreground">
                      {server.status}
                    </span>
                    <span
                      aria-hidden
                      className={cn(
                        "size-2 shrink-0 rounded-full",
                        server.status === "connected"
                          ? "bg-success"
                          : server.status === "failed"
                            ? "bg-destructive"
                            : server.status === "needs-auth"
                              ? "bg-warning"
                              : "bg-muted-foreground/50",
                      )}
                    />
                  </Row>
                ))
              )}
              <Row icon={<ServerCogIcon />} onClick={() => open("/settings/integrations")}>
                {gt("Manage connectors")}
              </Row>
            </>
          ) : null}
          {props.panel === "integrations" && !props.runClaudeCommand ? (
            <>
              <Row icon={<BlocksIcon />} onClick={() => open("/settings/integrations")}>
                {gt("Browse and manage integrations")}
              </Row>
              <Row icon={<PlugIcon />} onClick={() => open("/settings/providers")}>
                {gt("Providers and agents")}
              </Row>
              <Row icon={<KeyRoundIcon />} onClick={() => open("/settings/secrets")}>
                {gt("Secrets")}
              </Row>
            </>
          ) : null}
        </div>
      </ComposerBanner.Root>
    </div>
  );
}
