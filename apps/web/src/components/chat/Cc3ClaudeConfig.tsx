// CC3: GUI for the files and commands behind Claude Code's /permissions, /hooks, /agents, /memory,
// /output-style, /config, /login and /doctor. Reads and writes go through the CLI command runner.
import { useGT } from "gt-react";
import { useEffect, useState } from "react";

import { useStableCallback } from "../../hooks/useStableCallback";
import { Button } from "../ui/button";
import { Input } from "../ui/input";
import type { Cc3RunClaudeCommand } from "./Cc3ClaudeManager";

export const CONFIG_TABS = [
  "permissions",
  "hooks",
  "agents",
  "commands",
  "styles",
  "memory",
  "settings",
  "account",
] as const;
export type ConfigTab = (typeof CONFIG_TABS)[number];

type Scope = "user" | "project" | "local";
type Kind = "settings" | "memory" | "agent" | "command" | "style";
type Settings = Record<string, unknown>;

const SCOPES: ReadonlyArray<Scope> = ["user", "project", "local"];
const HOOK_EVENTS = [
  "PreToolUse",
  "PostToolUse",
  "UserPromptSubmit",
  "Stop",
  "SubagentStop",
  "SessionStart",
  "SessionEnd",
  "Notification",
  "PreCompact",
];
const PERMISSION_LISTS = ["allow", "ask", "deny"] as const;
const PERMISSION_MODES = ["default", "acceptEdits", "plan", "bypassPermissions"];

const selectClass = "h-7 rounded-md border border-input bg-popover px-1 text-xs";

async function call(
  run: Cc3RunClaudeCommand,
  request: {
    op: "read" | "write" | "list" | "delete";
    kind: Kind;
    scope: Scope;
    name?: string;
    text?: string;
  },
): Promise<string> {
  return (await run(`/cc3-config ${JSON.stringify(request)}`)).output;
}

function parseSettings(text: string): Settings {
  try {
    const value: unknown = JSON.parse(text);
    return value !== null && typeof value === "object" && !Array.isArray(value)
      ? (value as Settings)
      : {};
  } catch {
    return {};
  }
}

function ScopeSelect(props: { readonly value: Scope; readonly onChange: (scope: Scope) => void }) {
  return (
    <select
      className={selectClass}
      value={props.value}
      onChange={(event) => props.onChange(event.target.value as Scope)}
    >
      {SCOPES.map((scope) => (
        <option key={scope} value={scope}>
          {scope}
        </option>
      ))}
    </select>
  );
}

function Status(props: { readonly text: string }) {
  return props.text ? (
    <pre className="max-h-24 overflow-auto whitespace-pre-wrap font-mono text-xs text-muted-foreground">
      {props.text}
    </pre>
  ) : null;
}

/** The parsed settings file of one scope, with a save that writes the changed object back. */
function useSettingsFile(run: Cc3RunClaudeCommand, scope: Scope) {
  const [settings, setSettings] = useState<Settings>({});
  const [status, setStatus] = useState("");
  const load = useStableCallback(async (target: Scope) => {
    setSettings(parseSettings(await call(run, { op: "read", kind: "settings", scope: target })));
  });
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- loads the file when the scope changes.
  useEffect(() => {
    void load(scope);
  }, [scope, load]);
  const save = useStableCallback(async (next: Settings) => {
    setSettings(next);
    setStatus(
      await call(run, {
        op: "write",
        kind: "settings",
        scope,
        text: `${JSON.stringify(next, null, 2)}\n`,
      }).catch((error: unknown) => String(error)),
    );
  });
  return { settings, save, status };
}

function PermissionsTab(props: { readonly run: Cc3RunClaudeCommand }) {
  const gt = useGT();
  const [scope, setScope] = useState<Scope>("user");
  const [list, setList] = useState<(typeof PERMISSION_LISTS)[number]>("allow");
  const [rule, setRule] = useState("");
  const { settings, save, status } = useSettingsFile(props.run, scope);
  const permissions = (settings.permissions ?? {}) as Record<string, unknown>;
  const rules = (name: string) =>
    Array.isArray(permissions[name]) ? (permissions[name] as Array<string>) : [];
  const update = (patch: Record<string, unknown>) =>
    save({ ...settings, permissions: { ...permissions, ...patch } });
  return (
    <>
      <div className="flex flex-wrap items-center gap-1 text-xs">
        <ScopeSelect value={scope} onChange={setScope} />
        <span className="text-muted-foreground">{gt("Default mode")}</span>
        <select
          className={selectClass}
          value={String(permissions.defaultMode ?? "default")}
          onChange={(event) => void update({ defaultMode: event.target.value })}
        >
          {PERMISSION_MODES.map((mode) => (
            <option key={mode} value={mode}>
              {mode}
            </option>
          ))}
        </select>
      </div>
      {PERMISSION_LISTS.map((name) => (
        <div key={name} className="flex flex-col gap-0.5">
          <span className="text-xs font-semibold text-foreground">{name}</span>
          {rules(name).map((entry) => (
            <div key={entry} className="flex items-center gap-1 text-xs">
              <span className="min-w-0 flex-1 truncate font-mono">{entry}</span>
              <Button
                size="xs"
                variant="ghost-destructive"
                onClick={() =>
                  void update({ [name]: rules(name).filter((value) => value !== entry) })
                }
              >
                {gt("Remove")}
              </Button>
            </div>
          ))}
        </div>
      ))}
      <form
        className="flex gap-1"
        onSubmit={(event) => {
          event.preventDefault();
          const value = rule.trim();
          if (value === "" || rules(list).includes(value)) return;
          void update({ [list]: [...rules(list), value] }).then(() => setRule(""));
        }}
      >
        <select
          className={selectClass}
          value={list}
          onChange={(event) => setList(event.target.value as (typeof PERMISSION_LISTS)[number])}
        >
          {PERMISSION_LISTS.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
        <Input
          size="compact"
          value={rule}
          placeholder={gt("Rule, e.g. Bash(npm run test:*)")}
          onChange={(event) => setRule(event.target.value)}
        />
        <Button size="xs" type="submit" disabled={rule.trim() === ""}>
          {gt("Add rule")}
        </Button>
      </form>
      <Status text={status} />
    </>
  );
}

interface HookEntry {
  readonly event: string;
  readonly matcher: string;
  readonly command: string;
}

function flattenHooks(settings: Settings): ReadonlyArray<HookEntry> {
  const hooks = (settings.hooks ?? {}) as Record<
    string,
    ReadonlyArray<{ matcher?: string; hooks?: ReadonlyArray<{ command?: string }> }>
  >;
  return Object.entries(hooks).flatMap(([event, groups]) =>
    (Array.isArray(groups) ? groups : []).flatMap((group) =>
      (group.hooks ?? []).map((hook: { command?: string }) => ({
        event,
        matcher: group.matcher ?? "",
        command: hook.command ?? "",
      })),
    ),
  );
}

function buildHooks(entries: ReadonlyArray<HookEntry>): Record<string, unknown> {
  const hooks: Record<
    string,
    Array<{ matcher: string; hooks: Array<{ type: string; command: string }> }>
  > = {};
  for (const entry of entries) {
    const groups = (hooks[entry.event] ??= []);
    const group = groups.find((candidate) => candidate.matcher === entry.matcher);
    const hook = { type: "command", command: entry.command };
    if (group) group.hooks.push(hook);
    else groups.push({ matcher: entry.matcher, hooks: [hook] });
  }
  return hooks;
}

function HooksTab(props: { readonly run: Cc3RunClaudeCommand }) {
  const gt = useGT();
  const [scope, setScope] = useState<Scope>("user");
  const [event, setEvent] = useState(HOOK_EVENTS[0]!);
  const [matcher, setMatcher] = useState("");
  const [command, setCommand] = useState("");
  const { settings, save, status } = useSettingsFile(props.run, scope);
  const entries = flattenHooks(settings);
  const write = (next: ReadonlyArray<HookEntry>) => save({ ...settings, hooks: buildHooks(next) });
  return (
    <>
      <ScopeSelect value={scope} onChange={setScope} />
      {entries.length === 0 ? (
        <p className="text-xs text-muted-foreground">{gt("No hooks configured.")}</p>
      ) : null}
      {entries.map((entry, index) => (
        <div
          key={`${entry.event}:${entry.matcher}:${entry.command}`}
          className="flex items-center gap-1 text-xs"
        >
          <span className="shrink-0 font-medium text-foreground">{entry.event}</span>
          {entry.matcher ? (
            <span className="shrink-0 text-muted-foreground">{entry.matcher}</span>
          ) : null}
          <span className="min-w-0 flex-1 truncate font-mono">{entry.command}</span>
          <Button
            size="xs"
            variant="ghost-destructive"
            onClick={() => void write(entries.filter((_, at) => at !== index))}
          >
            {gt("Remove")}
          </Button>
        </div>
      ))}
      <form
        className="flex flex-wrap gap-1"
        onSubmit={(submit) => {
          submit.preventDefault();
          if (command.trim() === "") return;
          void write([
            ...entries,
            { event, matcher: matcher.trim(), command: command.trim() },
          ]).then(() => setCommand(""));
        }}
      >
        <select
          className={selectClass}
          value={event}
          onChange={(change) => setEvent(change.target.value)}
        >
          {HOOK_EVENTS.map((name) => (
            <option key={name} value={name}>
              {name}
            </option>
          ))}
        </select>
        <Input
          size="compact"
          className="w-28"
          value={matcher}
          placeholder={gt("Matcher, e.g. Bash")}
          onChange={(change) => setMatcher(change.target.value)}
        />
        <Input
          size="compact"
          className="min-w-40 flex-1"
          value={command}
          placeholder={gt("Shell command")}
          onChange={(change) => setCommand(change.target.value)}
        />
        <Button size="xs" type="submit" disabled={command.trim() === ""}>
          {gt("Add hook")}
        </Button>
      </form>
      <Status text={status} />
    </>
  );
}

const TEMPLATES: Record<"agent" | "command" | "style", (name: string) => string> = {
  agent: (name) =>
    `---\nname: ${name}\ndescription: When to use this agent\ntools: Read, Grep, Glob\n---\n\nYou are ${name}.\n`,
  command: () =>
    `---\ndescription: What this command does\n---\n\nPrompt text. Use $ARGUMENTS for input.\n`,
  style: (name) =>
    `---\nname: ${name}\ndescription: How Claude should respond\n---\n\nInstructions.\n`,
};

function FilesTab(props: {
  readonly run: Cc3RunClaudeCommand;
  readonly kind: "agent" | "command" | "style";
}) {
  const gt = useGT();
  const { run, kind } = props;
  const [scope, setScope] = useState<Scope>("user");
  const [names, setNames] = useState<ReadonlyArray<string>>([]);
  const [selected, setSelected] = useState<string | null>(null);
  const [text, setText] = useState("");
  const [fresh, setFresh] = useState("");
  const [status, setStatus] = useState("");
  const settingsFile = useSettingsFile(run, scope);
  const changeScope = (next: Scope) => {
    setScope(next);
    setSelected(null);
  };
  const loadNames = useStableCallback(async (target: Scope) => {
    try {
      setNames(JSON.parse(await call(run, { op: "list", kind, scope: target })) as Array<string>);
    } catch {
      setNames([]);
    }
  });
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- loads the file list when the scope changes.
  useEffect(() => {
    void loadNames(scope);
  }, [scope, loadNames]);
  const open = async (name: string, content?: string) => {
    setSelected(name);
    setText(content ?? (await call(run, { op: "read", kind, scope, name })));
  };
  const run2 = async (action: () => Promise<string>) => {
    setStatus(await action().catch((error: unknown) => String(error)));
    await loadNames(scope);
  };
  const activeStyle = String(settingsFile.settings.outputStyle ?? "default");
  return (
    <>
      <div className="flex flex-wrap items-center gap-1">
        <ScopeSelect value={scope} onChange={changeScope} />
        <form
          className="flex gap-1"
          onSubmit={(event) => {
            event.preventDefault();
            const name = fresh.trim();
            if (name === "") return;
            setFresh("");
            void open(name, TEMPLATES[kind](name));
          }}
        >
          <Input
            size="compact"
            className="w-32"
            value={fresh}
            placeholder={gt("New name")}
            onChange={(event) => setFresh(event.target.value)}
          />
          <Button size="xs" type="submit" disabled={fresh.trim() === ""}>
            {gt("New")}
          </Button>
        </form>
      </div>
      <div className="flex flex-wrap gap-1">
        {names.map((name) => (
          <Button
            key={name}
            size="xs"
            variant={selected === name ? "secondary" : "outline"}
            onClick={() => void open(name)}
          >
            {name}
            {kind === "style" && activeStyle === name ? " ✓" : ""}
          </Button>
        ))}
        {names.length === 0 ? (
          <span className="text-xs text-muted-foreground">{gt("Nothing here yet.")}</span>
        ) : null}
      </div>
      {kind === "style" ? (
        <div className="flex items-center gap-1 text-xs">
          <span className="text-muted-foreground">{gt("Active style")}</span>
          <select
            className={selectClass}
            value={activeStyle}
            onChange={(event) =>
              void settingsFile.save({ ...settingsFile.settings, outputStyle: event.target.value })
            }
          >
            {["default", "Explanatory", "Learning", ...names].map((name) => (
              <option key={name} value={name}>
                {name}
              </option>
            ))}
          </select>
        </div>
      ) : null}
      {selected !== null ? (
        <>
          <textarea
            className="h-40 w-full resize-y rounded-md border border-input bg-popover p-2 font-mono text-xs"
            value={text}
            onChange={(event) => setText(event.target.value)}
          />
          <div className="flex gap-1">
            <Button
              size="xs"
              onClick={() =>
                void run2(() => call(run, { op: "write", kind, scope, name: selected, text }))
              }
            >
              {gt("Save")}
            </Button>
            <Button
              size="xs"
              variant="destructive-outline"
              onClick={() =>
                void run2(async () => {
                  const result = await call(run, { op: "delete", kind, scope, name: selected });
                  setSelected(null);
                  return result;
                })
              }
            >
              {gt("Delete")}
            </Button>
          </div>
        </>
      ) : null}
      <Status text={status} />
    </>
  );
}

/** One file as text: the memory file or the raw settings JSON of a scope. */
function SingleFileTab(props: {
  readonly run: Cc3RunClaudeCommand;
  readonly kind: "memory" | "settings";
}) {
  const gt = useGT();
  const { run, kind } = props;
  const [scope, setScope] = useState<Scope>("user");
  const [text, setText] = useState("");
  const [status, setStatus] = useState("");
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- loads the file when the scope changes.
  useEffect(() => {
    void call(run, { op: "read", kind, scope }).then(setText);
  }, [run, kind, scope]);
  return (
    <>
      <ScopeSelect value={scope} onChange={setScope} />
      <textarea
        className="h-48 w-full resize-y rounded-md border border-input bg-popover p-2 font-mono text-xs"
        value={text}
        onChange={(event) => setText(event.target.value)}
      />
      <Button
        size="xs"
        className="self-start"
        onClick={() =>
          void call(run, { op: "write", kind, scope, text })
            .then(setStatus)
            .catch((error: unknown) => setStatus(String(error)))
        }
      >
        {gt("Save")}
      </Button>
      <Status text={status} />
    </>
  );
}

function AccountTab(props: { readonly run: Cc3RunClaudeCommand }) {
  const gt = useGT();
  const { run } = props;
  const [output, setOutput] = useState("");
  const [busy, setBusy] = useState(false);
  const go = useStableCallback(async (command: string) => {
    setBusy(true);
    setOutput(
      await run(command)
        .then((result) => result.output)
        .catch((error: unknown) => String(error)),
    );
    setBusy(false);
  });
  // oxlint-disable-next-line t3code/no-unreasoned-state-sync -- shows the sign-in state when the tab opens.
  useEffect(() => {
    void go("/cc3-cli auth status");
  }, [go]);
  return (
    <>
      <div className="flex flex-wrap gap-1">
        <Button
          size="xs"
          variant="outline"
          disabled={busy}
          onClick={() => void go("/cc3-cli auth status")}
        >
          {gt("Status")}
        </Button>
        <Button size="xs" disabled={busy} onClick={() => void go("/cc3-cli auth login --claudeai")}>
          {gt("Sign in (opens browser)")}
        </Button>
        <Button
          size="xs"
          variant="outline"
          disabled={busy}
          onClick={() => void go("/cc3-cli auth logout")}
        >
          {gt("Sign out")}
        </Button>
        <Button
          size="xs"
          variant="outline"
          disabled={busy}
          onClick={() => void go("/cc3-cli doctor")}
        >
          {gt("Run doctor")}
        </Button>
        <Button
          size="xs"
          variant="outline"
          disabled={busy}
          onClick={() => void go("/cc3-cli update")}
        >
          {gt("Update Claude Code")}
        </Button>
      </div>
      {busy ? <span className="text-xs text-muted-foreground">{gt("Working...")}</span> : null}
      <Status text={output} />
    </>
  );
}

export function Cc3ClaudeConfig(props: {
  readonly run: Cc3RunClaudeCommand;
  readonly initialTab: ConfigTab;
}) {
  const gt = useGT();
  const [tab, setTab] = useState<ConfigTab>(props.initialTab);
  const labels: Record<ConfigTab, string> = {
    permissions: gt("Permissions"),
    hooks: gt("Hooks"),
    agents: gt("Agents"),
    commands: gt("Commands"),
    styles: gt("Output styles"),
    memory: gt("Memory"),
    settings: gt("Settings JSON"),
    account: gt("Account"),
  };
  return (
    <div className="flex flex-col gap-2 px-2.5 pb-1">
      <div className="flex flex-wrap gap-1">
        {CONFIG_TABS.map((name) => (
          <Button
            key={name}
            size="xs"
            variant={tab === name ? "secondary" : "ghost-muted"}
            onClick={() => setTab(name)}
          >
            {labels[name]}
          </Button>
        ))}
      </div>
      {tab === "permissions" ? <PermissionsTab run={props.run} /> : null}
      {tab === "hooks" ? <HooksTab run={props.run} /> : null}
      {tab === "agents" ? <FilesTab key="agent" run={props.run} kind="agent" /> : null}
      {tab === "commands" ? <FilesTab key="command" run={props.run} kind="command" /> : null}
      {tab === "styles" ? <FilesTab key="style" run={props.run} kind="style" /> : null}
      {tab === "memory" ? <SingleFileTab key="memory" run={props.run} kind="memory" /> : null}
      {tab === "settings" ? <SingleFileTab key="settings" run={props.run} kind="settings" /> : null}
      {tab === "account" ? <AccountTab run={props.run} /> : null}
    </div>
  );
}
