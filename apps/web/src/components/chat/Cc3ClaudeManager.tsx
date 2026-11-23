// CC3: GUI for `claude plugin ...` and `claude mcp ...`. Every action runs the real CLI subcommand
// through the provider's CLI command runner and reloads the list from it.
import { useGT } from "gt-react";
import { useEffect, useState } from "react";

import { useStableCallback } from "../../hooks/useStableCallback";

import { Button } from "../ui/button";
import { Input } from "../ui/input";

export interface Cc3ClaudeCommandResult {
  readonly output: string;
  readonly mcpServers?: ReadonlyArray<{ readonly name: string; readonly status: string }>;
}

export type Cc3RunClaudeCommand = (command: string) => Promise<Cc3ClaudeCommandResult>;

interface InstalledPlugin {
  readonly id: string;
  readonly version?: string;
  readonly scope?: string;
  readonly enabled: boolean;
}
interface Marketplace {
  readonly name: string;
  readonly source?: string;
  readonly repo?: string;
}
interface CatalogPlugin {
  readonly id: string;
  readonly name: string;
  readonly marketplace: string;
  readonly description: string;
}

function parseJsonList<T>(text: string): ReadonlyArray<T> {
  try {
    const value: unknown = JSON.parse(text);
    return Array.isArray(value) ? (value as ReadonlyArray<T>) : [];
  } catch {
    return [];
  }
}

function useCommandRunner(run: Cc3RunClaudeCommand, reload: () => Promise<void>) {
  const [busy, setBusy] = useState<string | null>(null);
  const [message, setMessage] = useState("");
  const act = useStableCallback(async (command: string) => {
    setBusy(command);
    try {
      setMessage((await run(command)).output);
    } catch (error) {
      setMessage(error instanceof Error ? error.message : String(error));
    }
    await reload();
    setBusy(null);
  });
  return { busy, message, act };
}

const TABS = ["discover", "installed", "marketplaces"] as const;
type Tab = (typeof TABS)[number];

export function Cc3PluginsManager(props: { readonly run: Cc3RunClaudeCommand }) {
  const gt = useGT();
  const { run } = props;
  const [tab, setTab] = useState<Tab>("installed");
  const [installed, setInstalled] = useState<ReadonlyArray<InstalledPlugin>>([]);
  const [marketplaces, setMarketplaces] = useState<ReadonlyArray<Marketplace>>([]);
  const [catalog, setCatalog] = useState<ReadonlyArray<CatalogPlugin>>([]);
  const [query, setQuery] = useState("");
  const [source, setSource] = useState("");
  const [loading, setLoading] = useState(true);

  const reload = useStableCallback(async () => {
    const [plugins, markets, available] = await Promise.all([
      run("/plugin list --json"),
      run("/plugin marketplace list --json"),
      run("/plugin catalog"),
    ]);
    setInstalled(parseJsonList<InstalledPlugin>(plugins.output));
    setMarketplaces(parseJsonList<Marketplace>(markets.output));
    setCatalog(parseJsonList<CatalogPlugin>(available.output));
    setLoading(false);
  });
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- loads the lists from the CLI when the panel opens.
  useEffect(() => {
    void reload();
  }, [reload]);
  const { busy, message, act } = useCommandRunner(run, reload);

  const installedIds = new Set(installed.map((plugin) => plugin.id));
  const needle = query.trim().toLowerCase();
  const matches = catalog
    .filter(
      (plugin) =>
        needle === "" ||
        plugin.name.toLowerCase().includes(needle) ||
        plugin.description.toLowerCase().includes(needle),
    )
    .slice(0, 40);
  const label = (name: Tab) =>
    name === "discover"
      ? gt("Discover")
      : name === "installed"
        ? gt("Installed")
        : gt("Marketplaces");

  return (
    <div className="flex flex-col gap-2 px-2.5 pb-1">
      <div className="flex gap-1">
        {TABS.map((name) => (
          <Button
            key={name}
            size="xs"
            variant={tab === name ? "secondary" : "ghost-muted"}
            onClick={() => setTab(name)}
          >
            {label(name)}
          </Button>
        ))}
        {loading || busy ? (
          <span className="ms-auto self-center text-xs text-muted-foreground">
            {gt("Working...")}
          </span>
        ) : null}
      </div>
      {tab === "discover" ? (
        <>
          <Input
            size="compact"
            value={query}
            placeholder={gt("Search plugins")}
            onChange={(event) => setQuery(event.target.value)}
          />
          {matches.map((plugin) => (
            <div key={plugin.id} className="flex items-start gap-2 text-xs">
              <div className="min-w-0 flex-1">
                <div className="truncate font-medium text-foreground">
                  {plugin.name}
                  <span className="ms-1 font-normal text-muted-foreground">
                    {plugin.marketplace}
                  </span>
                </div>
                <div className="line-clamp-2 text-muted-foreground">{plugin.description}</div>
              </div>
              {installedIds.has(plugin.id) ? (
                <span className="shrink-0 text-muted-foreground">{gt("Installed")}</span>
              ) : (
                <Button
                  size="xs"
                  disabled={busy !== null}
                  onClick={() => void act(`/plugin install ${plugin.id}`)}
                >
                  {gt("Install")}
                </Button>
              )}
            </div>
          ))}
          {!loading && matches.length === 0 ? (
            <p className="text-xs text-muted-foreground">{gt("No plugins found.")}</p>
          ) : null}
        </>
      ) : null}
      {tab === "installed" ? (
        <>
          {!loading && installed.length === 0 ? (
            <p className="text-xs text-muted-foreground">{gt("No plugins installed.")}</p>
          ) : null}
          {installed.map((plugin) => (
            <div key={`${plugin.id}:${plugin.scope}`} className="flex items-center gap-1 text-xs">
              <span className="min-w-0 flex-1 truncate font-medium text-foreground">
                {plugin.id}
                <span className="ms-1 font-normal text-muted-foreground">
                  {plugin.version} {plugin.scope}
                </span>
              </span>
              <Button
                size="xs"
                variant="outline"
                disabled={busy !== null}
                onClick={() =>
                  void act(`/plugin ${plugin.enabled ? "disable" : "enable"} ${plugin.id}`)
                }
              >
                {plugin.enabled ? gt("Disable") : gt("Enable")}
              </Button>
              <Button
                size="xs"
                variant="outline"
                disabled={busy !== null}
                onClick={() => void act(`/plugin update ${plugin.id}`)}
              >
                {gt("Update")}
              </Button>
              <Button
                size="xs"
                variant="destructive-outline"
                disabled={busy !== null}
                onClick={() => void act(`/plugin uninstall ${plugin.id}`)}
              >
                {gt("Remove")}
              </Button>
            </div>
          ))}
        </>
      ) : null}
      {tab === "marketplaces" ? (
        <>
          {marketplaces.map((market) => (
            <div key={market.name} className="flex items-center gap-1 text-xs">
              <span className="min-w-0 flex-1 truncate font-medium text-foreground">
                {market.name}
                <span className="ms-1 font-normal text-muted-foreground">
                  {market.repo ?? market.source}
                </span>
              </span>
              <Button
                size="xs"
                variant="outline"
                disabled={busy !== null}
                onClick={() => void act(`/plugin marketplace update ${market.name}`)}
              >
                {gt("Update")}
              </Button>
              <Button
                size="xs"
                variant="destructive-outline"
                disabled={busy !== null}
                onClick={() => void act(`/plugin marketplace remove ${market.name}`)}
              >
                {gt("Remove")}
              </Button>
            </div>
          ))}
          <form
            className="flex gap-1"
            onSubmit={(event) => {
              event.preventDefault();
              if (source.trim() === "") return;
              void act(`/plugin marketplace add ${source.trim()}`).then(() => setSource(""));
            }}
          >
            <Input
              size="compact"
              value={source}
              placeholder={gt("GitHub repo, URL or path")}
              onChange={(event) => setSource(event.target.value)}
            />
            <Button size="xs" type="submit" disabled={busy !== null || source.trim() === ""}>
              {gt("Add marketplace")}
            </Button>
          </form>
        </>
      ) : null}
      {message ? (
        <pre className="max-h-24 overflow-auto whitespace-pre-wrap font-mono text-xs text-muted-foreground">
          {message}
        </pre>
      ) : null}
    </div>
  );
}

type McpServer = { readonly name: string; readonly status: string };
type Transport = "stdio" | "http" | "sse";
type Scope = "local" | "user" | "project";

export function Cc3McpManager(props: { readonly run: Cc3RunClaudeCommand }) {
  const gt = useGT();
  const { run } = props;
  const [servers, setServers] = useState<ReadonlyArray<McpServer>>([]);
  const [loading, setLoading] = useState(true);
  const [name, setName] = useState("");
  const [target, setTarget] = useState("");
  const [transport, setTransport] = useState<Transport>("http");
  const [scope, setScope] = useState<Scope>("user");

  const reload = useStableCallback(async () => {
    setServers((await run("/mcp")).mcpServers ?? []);
    setLoading(false);
  });
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- loads the server list from the CLI when the panel opens.
  useEffect(() => {
    void reload();
  }, [reload]);
  const { busy, message, act } = useCommandRunner(run, reload);

  const canAdd = name.trim() !== "" && target.trim() !== "" && busy === null;
  const addCommand =
    transport === "stdio"
      ? `/mcp add -s ${scope} ${name.trim()} -- ${target.trim()}`
      : `/mcp add -s ${scope} --transport ${transport} ${name.trim()} ${target.trim()}`;
  return (
    <div className="flex flex-col gap-2 px-2.5 pb-1">
      {loading || busy ? (
        <span className="text-xs text-muted-foreground">{gt("Working...")}</span>
      ) : null}
      {!loading && servers.length === 0 ? (
        <p className="text-xs text-muted-foreground">{gt("No MCP servers configured.")}</p>
      ) : null}
      {servers.map((server) => (
        <div key={server.name} className="flex items-center gap-2 text-xs">
          <span className="min-w-0 flex-1 truncate font-medium text-foreground">{server.name}</span>
          <span className="shrink-0 text-muted-foreground">{server.status}</span>
          <Button
            size="xs"
            variant="destructive-outline"
            disabled={busy !== null}
            onClick={() => void act(`/mcp remove ${server.name}`)}
          >
            {gt("Remove")}
          </Button>
        </div>
      ))}
      <form
        className="flex flex-wrap gap-1"
        onSubmit={(event) => {
          event.preventDefault();
          if (canAdd) void act(addCommand).then(() => (setName(""), setTarget("")));
        }}
      >
        <select
          className="h-7 rounded-md border border-input bg-popover px-1 text-xs"
          value={transport}
          onChange={(event) => setTransport(event.target.value as Transport)}
        >
          <option value="http">http</option>
          <option value="sse">sse</option>
          <option value="stdio">stdio</option>
        </select>
        <select
          className="h-7 rounded-md border border-input bg-popover px-1 text-xs"
          value={scope}
          onChange={(event) => setScope(event.target.value as Scope)}
        >
          <option value="user">user</option>
          <option value="project">project</option>
          <option value="local">local</option>
        </select>
        <Input
          size="compact"
          className="w-28"
          value={name}
          placeholder={gt("Name")}
          onChange={(event) => setName(event.target.value)}
        />
        <Input
          size="compact"
          className="min-w-40 flex-1"
          value={target}
          placeholder={transport === "stdio" ? gt("Command and arguments") : gt("Server URL")}
          onChange={(event) => setTarget(event.target.value)}
        />
        <Button size="xs" type="submit" disabled={!canAdd}>
          {gt("Add server")}
        </Button>
      </form>
      {message ? (
        <pre className="max-h-24 overflow-auto whitespace-pre-wrap font-mono text-xs text-muted-foreground">
          {message}
        </pre>
      ) : null}
    </div>
  );
}
