// CC3: slash commands Claude Code runs in its terminal UI that have no meaning inside a headless
// session. CC3 answers them itself by opening the matching screen, so `/mcp`, `/context`,
// `/usage` and friends work like they do in Claude Code. Provider commands (`/compact`,
// `/init`, skills...) keep going to the provider.
/** Popups drawn above the composer, see Cc3CommandPanel. */
export type Cc3CommandPanelKind = "connectors" | "overview" | "integrations" | "cli";

export type Cc3LocalSlashAction =
  | { readonly kind: "panel"; readonly panel: Cc3CommandPanelKind }
  | { readonly kind: "route"; readonly to: string }
  | { readonly kind: "palette" };

export interface Cc3LocalSlashCommand {
  readonly name: string;
  readonly description: string;
  readonly action: Cc3LocalSlashAction;
}

const route = (to: string): Cc3LocalSlashAction => ({ kind: "route", to });

export const CC3_LOCAL_SLASH_COMMANDS = [
  {
    name: "mcp",
    description: "Show connectors (MCP servers) and their status",
    action: { kind: "panel", panel: "connectors" },
  },
  {
    name: "context",
    description: "Show how full the context window is",
    action: { kind: "panel", panel: "overview" },
  },
  {
    name: "status",
    description: "Show session status: folder, context and connectors",
    action: { kind: "panel", panel: "overview" },
  },
  { name: "usage", description: "Show plan usage and limits", action: route("/usage") },
  { name: "cost", description: "Show token usage and cost", action: route("/usage") },
  { name: "config", description: "Open settings", action: route("/settings/general") },
  { name: "settings", description: "Open settings", action: route("/settings/general") },
  {
    name: "permissions",
    description: "Manage access and permissions",
    action: route("/settings/access"),
  },
  { name: "memory", description: "Edit memory and chat defaults", action: route("/settings/chat") },
  { name: "login", description: "Sign in to a provider", action: route("/settings/providers") },
  { name: "logout", description: "Sign out of a provider", action: route("/settings/providers") },
  { name: "theme", description: "Change the theme", action: route("/settings/appearance") },
  { name: "doctor", description: "Run diagnostics", action: route("/settings/diagnostics") },
  {
    name: "marketplace",
    description: "Browse and manage plugins and integrations",
    action: { kind: "panel", panel: "integrations" },
  },
  {
    name: "agents",
    description: "Manage agents and integrations",
    action: { kind: "panel", panel: "integrations" },
  },
  {
    name: "plugins",
    description: "Manage plugins and integrations",
    action: { kind: "panel", panel: "integrations" },
  },
  {
    name: "hooks",
    description: "Manage hooks and integrations",
    action: { kind: "panel", panel: "integrations" },
  },
  {
    name: "terminal-setup",
    description: "Configure keyboard shortcuts",
    action: route("/settings/keybindings"),
  },
  {
    name: "vim",
    description: "Configure keyboard shortcuts",
    action: route("/settings/keybindings"),
  },
  { name: "help", description: "Search all commands and actions", action: { kind: "palette" } },
  { name: "resume", description: "Find and reopen a chat", action: { kind: "palette" } },
] as const satisfies ReadonlyArray<Cc3LocalSlashCommand>;

export type Cc3LocalSlashName = (typeof CC3_LOCAL_SLASH_COMMANDS)[number]["name"];

const BY_NAME = new Map<string, Cc3LocalSlashCommand>(
  CC3_LOCAL_SLASH_COMMANDS.map((command) => [command.name, command]),
);

export function isCc3LocalSlashName(name: string): name is Cc3LocalSlashName {
  return BY_NAME.has(name);
}

export function cc3LocalSlashCommand(name: string): Cc3LocalSlashCommand | null {
  return BY_NAME.get(name) ?? null;
}

/** `/mcp` or `/mcp extra words`, and only a command CC3 answers itself. */
export function parseCc3LocalSlash(prompt: string): Cc3LocalSlashCommand | null {
  const match = /^\/([a-z][a-z-]*)(?:\s.*)?$/i.exec(prompt.trim());
  return match?.[1] ? cc3LocalSlashCommand(match[1].toLowerCase()) : null;
}
