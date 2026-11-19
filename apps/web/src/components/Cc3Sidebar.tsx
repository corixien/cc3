// CC3: the thread sidebar. A flat list of chats (title and a live status only) that can be sorted
// into collapsible folders. Replaces the project-grouped upstream sidebar in the app layout.
import { scopeThreadRef } from "@t3tools/client-runtime/environment";
import {
  threadRuntimeIsActive,
  type EnvironmentThreadShell,
} from "@t3tools/client-runtime/state/shell";
import { settlePromise } from "@t3tools/client-runtime/state/runtime";
import { useNavigate, useParams } from "@tanstack/react-router";
import * as Schema from "effect/Schema";
import { useGT } from "gt-react";
import {
  ArrowUpDownIcon,
  ChevronDownIcon,
  ChevronRightIcon,
  PlusIcon,
  SearchIcon,
  XIcon,
} from "lucide-react";
import { useMemo, useRef, useState } from "react";

import { isElectron } from "~/env";
import { useLocalStorage } from "../hooks/useLocalStorage";
import { useScratchProject } from "../hooks/useScratchProject";
import { useThreadActions } from "../hooks/useThreadActions";
import { cn } from "../lib/utils";
import { readLocalApi } from "../localApi";
import { useThreadShells } from "../state/entities";
import { buildThreadRouteParams } from "../threadRoutes";
import { SidebarChromeFooter, SidebarChromeHeader } from "./sidebar/SidebarChrome";
import {
  Menu,
  MenuCheckboxItem,
  MenuGroup,
  MenuGroupLabel,
  MenuItem,
  MenuPopup,
  MenuRadioGroup,
  MenuRadioItem,
  MenuSeparator,
  MenuSub,
  MenuSubPopup,
  MenuSubTrigger,
  MenuTrigger,
} from "./ui/menu";
import { SidebarContent } from "./ui/sidebar";
import { Spinner } from "./ui/spinner";

const SortMode = Schema.Literals(["recent", "alpha", "manual"]);
type SortMode = typeof SortMode.Type;
const FolderState = Schema.Struct({
  folders: Schema.Array(
    Schema.Struct({ id: Schema.String, name: Schema.String, collapsed: Schema.Boolean }),
  ),
  assignments: Schema.Record(Schema.String, Schema.String),
  /** Chat keys in the user's own order (used by manual sorting). */
  order: Schema.optionalKey(Schema.Array(Schema.String)),
  sort: Schema.optionalKey(SortMode),
  /** Keep groups in the user's own order while chats inside them sort by `sort`. */
  groupsManual: Schema.optionalKey(Schema.Boolean),
});
type FolderState = typeof FolderState.Type;
const EMPTY_FOLDERS: FolderState = { folders: [], assignments: {} };
const STORAGE_KEY = "cc3:sidebar-folders";

const threadKey = (thread: EnvironmentThreadShell) => `${thread.environmentId}:${thread.id}`;
const activityOf = (thread: EnvironmentThreadShell) =>
  Date.parse(thread.latestUserMessageAt ?? thread.updatedAt) || 0;

type ChatStatus = "needs-input" | "working" | null;
function statusOf(thread: EnvironmentThreadShell): ChatStatus {
  if (thread.hasPendingApprovals || thread.hasPendingUserInput) return "needs-input";
  if (threadRuntimeIsActive(thread.runtime)) return "working";
  return null;
}

function StatusMark({ status }: { status: ChatStatus }) {
  if (status === "working") return <Spinner size="sm" aria-label="Working" className="shrink-0" />;
  if (status === "needs-input") {
    return (
      <span
        aria-label="Needs input"
        className="size-2 shrink-0 animate-pulse rounded-full bg-warning"
      />
    );
  }
  return null;
}

function sortThreads(
  threads: readonly EnvironmentThreadShell[],
  mode: SortMode,
  order: readonly string[],
): EnvironmentThreadShell[] {
  const byRecent = (a: EnvironmentThreadShell, b: EnvironmentThreadShell) =>
    activityOf(b) - activityOf(a);
  if (mode === "alpha") {
    return threads.toSorted(
      (a, b) =>
        a.title.localeCompare(b.title, undefined, { sensitivity: "base" }) || byRecent(a, b),
    );
  }
  if (mode === "manual") {
    const index = new Map(order.map((key, position) => [key, position]));
    return threads.toSorted((a, b) => {
      const left = index.get(threadKey(a));
      const right = index.get(threadKey(b));
      if (left !== undefined && right !== undefined) return left - right;
      if (left !== undefined) return -1;
      if (right !== undefined) return 1;
      return byRecent(a, b);
    });
  }
  return threads.toSorted(byRecent);
}

export default function Cc3Sidebar() {
  const gt = useGT();
  const navigate = useNavigate();
  const params = useParams({ strict: false }) as { environmentId?: string; threadId?: string };
  const shells = useThreadShells();
  const { archiveThread, deleteThread } = useThreadActions();
  const { scratchEnvironmentId, startScratchThread } = useScratchProject();
  const [state, setState] = useLocalStorage(STORAGE_KEY, EMPTY_FOLDERS, FolderState);
  const [editing, setEditing] = useState<string | null>(null);
  const drag = useRef<{ kind: "chat" | "folder"; id: string } | null>(null);
  const [query, setQuery] = useState("");
  // The right-click menu is drawn with the app's own menu so it matches the rest of CC3.
  const [menu, setMenu] = useState<
    | { x: number; y: number; kind: "thread"; thread: EnvironmentThreadShell }
    | { x: number; y: number; kind: "folder"; folderId: string }
    | null
  >(null);

  const sort = state.sort ?? "recent";
  const order = state.order ?? [];
  const groupsManual = state.groupsManual === true || sort === "manual";

  const visible = useMemo(
    () =>
      shells
        .filter((thread) => thread.archivedAt === null && thread.deletedAt === null)
        .filter((thread) => thread.title.toLowerCase().includes(query.trim().toLowerCase())),
    [shells, query],
  );
  const folderIds = new Set(state.folders.map((folder) => folder.id));
  const folderOf = (thread: EnvironmentThreadShell) => {
    const id = state.assignments[threadKey(thread)];
    return id !== undefined && folderIds.has(id) ? id : null;
  };
  const loose = sortThreads(
    visible.filter((thread) => folderOf(thread) === null),
    sort,
    order,
  );
  const folderThreads = (id: string) =>
    sortThreads(
      visible.filter((thread) => folderOf(thread) === id),
      sort,
      order,
    );
  const folders = groupsManual
    ? state.folders
    : state.folders.toSorted((a, b) => {
        if (sort === "alpha")
          return a.name.localeCompare(b.name, undefined, { sensitivity: "base" });
        const latest = (id: string) => Math.max(0, ...folderThreads(id).map(activityOf));
        return latest(b.id) - latest(a.id);
      });

  const assign = (key: string, folderId: string | null) =>
    setState((current) => {
      const assignments = { ...current.assignments };
      if (folderId === null) delete assignments[key];
      else assignments[key] = folderId;
      return { ...current, assignments };
    });
  const addFolder = (threadToAssign?: EnvironmentThreadShell) => {
    const id = crypto.randomUUID();
    setState((current) => ({
      ...current,
      folders: [...current.folders, { id, name: gt("New folder"), collapsed: false }],
      assignments: threadToAssign
        ? { ...current.assignments, [threadKey(threadToAssign)]: id }
        : current.assignments,
    }));
    setEditing(id);
  };
  const patchFolder = (id: string, patch: Partial<FolderState["folders"][number]>) =>
    setState((current) => ({
      ...current,
      folders: current.folders.map((folder) =>
        folder.id === id ? { ...folder, ...patch } : folder,
      ),
    }));
  const removeFolder = (id: string) =>
    setState((current) => ({
      ...current,
      folders: current.folders.filter((folder) => folder.id !== id),
      assignments: Object.fromEntries(
        Object.entries(current.assignments).filter(([, folderId]) => folderId !== id),
      ),
    }));
  /** Dropping a chat on another chat puts it before that chat, in that chat's group. */
  const dropChatOn = (target: EnvironmentThreadShell) => {
    const dragged = drag.current;
    drag.current = null;
    if (dragged?.kind !== "chat" || dragged.id === threadKey(target)) return;
    setState((current) => {
      const assignments = { ...current.assignments };
      const targetFolder = assignments[threadKey(target)];
      if (targetFolder === undefined) delete assignments[dragged.id];
      else assignments[dragged.id] = targetFolder;
      // The current on-screen order becomes the manual order, then the dragged chat moves.
      const all = [...folderIds, null].flatMap((id) =>
        (id === null ? loose : folderThreads(id)).map(threadKey),
      );
      const next = all.filter((key) => key !== dragged.id);
      next.splice(Math.max(0, next.indexOf(threadKey(target))), 0, dragged.id);
      return { ...current, assignments, order: next, sort: "manual" };
    });
  };
  const dropFolderOn = (targetId: string) => {
    const dragged = drag.current;
    drag.current = null;
    if (dragged?.kind !== "folder" || dragged.id === targetId) return;
    setState((current) => {
      const ordered = folders.filter((folder) => folder.id !== dragged.id);
      const moving = current.folders.find((folder) => folder.id === dragged.id);
      if (!moving) return current;
      ordered.splice(
        Math.max(
          0,
          ordered.findIndex((folder) => folder.id === targetId),
        ),
        0,
        moving,
      );
      return { ...current, folders: ordered, groupsManual: true };
    });
  };

  const open = (thread: EnvironmentThreadShell) =>
    void navigate({
      to: "/$environmentId/$threadId",
      params: buildThreadRouteParams(scopeThreadRef(thread.environmentId, thread.id)),
    });
  const homeEnvironmentId = scratchEnvironmentId(null);
  const newChat = () => {
    if (homeEnvironmentId !== null) void startScratchThread(homeEnvironmentId);
  };

  const archiveChat = async (thread: EnvironmentThreadShell) => {
    await archiveThread(scopeThreadRef(thread.environmentId, thread.id), {});
  };
  const deleteChat = async (thread: EnvironmentThreadShell) => {
    const api = readLocalApi();
    if (api) {
      const confirmed = await settlePromise(() =>
        api.dialogs.confirm(gt('Delete chat "{title}"?', { title: thread.title }), {
          variant: "destructive",
        }),
      );
      if (confirmed._tag === "Failure" || !confirmed.value) return;
    }
    await deleteThread(scopeThreadRef(thread.environmentId, thread.id));
  };

  const row = (thread: EnvironmentThreadShell) => {
    const selected = params.environmentId === thread.environmentId && params.threadId === thread.id;
    const status = statusOf(thread);
    return (
      <li key={threadKey(thread)}>
        <button
          type="button"
          draggable
          onDragStart={() => (drag.current = { kind: "chat", id: threadKey(thread) })}
          onDragOver={(event) => event.preventDefault()}
          onDrop={(event) => {
            event.stopPropagation();
            dropChatOn(thread);
          }}
          onClick={() => open(thread)}
          onContextMenu={(event) => {
            event.preventDefault();
            setMenu({ x: event.clientX, y: event.clientY, kind: "thread", thread });
          }}
          className={cn(
            "flex h-7 w-full items-center gap-2 rounded-md px-2.5 text-left text-[13px] text-sidebar-foreground/90 transition-colors hover:bg-sidebar-row-hover hover:text-sidebar-foreground",
            selected && "bg-sidebar-row-selected text-sidebar-foreground",
          )}
        >
          <span className="min-w-0 flex-1 truncate">{thread.title}</span>
          <StatusMark status={status} />
        </button>
      </li>
    );
  };

  return (
    <>
      <SidebarChromeHeader isElectron={isElectron} />
      <SidebarContent className="min-h-0 gap-0 px-2 pt-2">
        <div className="flex items-center gap-1 pb-3">
          <div className="flex h-8 min-w-0 flex-1 items-center gap-2 rounded-md px-2 text-sm text-sidebar-muted-foreground hover:bg-sidebar-row-hover focus-within:bg-sidebar-row-hover">
            <SearchIcon className="size-4 shrink-0" />
            <input
              value={query}
              onChange={(event) => setQuery(event.currentTarget.value)}
              placeholder={gt("Search", { $context: "search field placeholder" })}
              aria-label={gt("Search chats")}
              className="min-w-0 flex-1 bg-transparent text-sidebar-foreground outline-none placeholder:text-sidebar-muted-foreground"
            />
            {query ? (
              <button type="button" aria-label={gt("Clear search")} onClick={() => setQuery("")}>
                <XIcon className="size-3.5" />
              </button>
            ) : null}
          </div>
          <Menu>
            <MenuTrigger
              render={
                <button
                  type="button"
                  aria-label={gt("Sort chats")}
                  title={gt("Sort chats")}
                  className="flex size-8 shrink-0 items-center justify-center rounded-md text-sidebar-muted-foreground transition-colors hover:bg-sidebar-row-hover hover:text-sidebar-foreground"
                />
              }
            >
              <ArrowUpDownIcon className="size-4" />
            </MenuTrigger>
            <MenuPopup align="end" side="bottom" className="min-w-52">
              <MenuGroup>
                <MenuGroupLabel>{gt("Sort chats")}</MenuGroupLabel>
                <MenuRadioGroup
                  value={sort}
                  onValueChange={(value) =>
                    setState((current) => ({ ...current, sort: value as SortMode }))
                  }
                >
                  <MenuRadioItem value="recent">{gt("Recently active")}</MenuRadioItem>
                  <MenuRadioItem value="alpha">{gt("Alphabetical")}</MenuRadioItem>
                  <MenuRadioItem value="manual">{gt("Manual (drag to reorder)")}</MenuRadioItem>
                </MenuRadioGroup>
              </MenuGroup>
              <MenuSeparator />
              <MenuCheckboxItem
                checked={groupsManual}
                disabled={sort === "manual"}
                onCheckedChange={(checked) =>
                  setState((current) => ({ ...current, groupsManual: checked }))
                }
              >
                {gt("Groups in my own order")}
              </MenuCheckboxItem>
            </MenuPopup>
          </Menu>
          <button
            type="button"
            aria-label={gt("New chat")}
            title={gt("New chat")}
            onClick={newChat}
            disabled={homeEnvironmentId === null}
            className="flex size-8 shrink-0 items-center justify-center rounded-md text-sidebar-muted-foreground transition-colors hover:bg-sidebar-row-hover hover:text-sidebar-foreground disabled:opacity-50"
          >
            <PlusIcon className="size-4" />
          </button>
        </div>
        {folders.map((folder) => {
          const inFolder = folderThreads(folder.id);
          const working = inFolder.some((thread) => statusOf(thread) !== null);
          return (
            <section
              key={folder.id}
              className="mb-4"
              onDragOver={(event) => event.preventDefault()}
              onDrop={() => {
                const dragged = drag.current;
                if (dragged?.kind === "chat") assign(dragged.id, folder.id);
                drag.current = null;
              }}
            >
              <div
                draggable={editing !== folder.id}
                onDragStart={() => (drag.current = { kind: "folder", id: folder.id })}
                onDragOver={(event) => event.preventDefault()}
                onDrop={(event) => {
                  if (drag.current?.kind === "folder") {
                    event.stopPropagation();
                    dropFolderOn(folder.id);
                  }
                }}
                onContextMenu={(event) => {
                  event.preventDefault();
                  setMenu({
                    x: event.clientX,
                    y: event.clientY,
                    kind: "folder",
                    folderId: folder.id,
                  });
                }}
                className="mb-1 flex h-8 items-center gap-1 rounded-md px-2.5 text-[15px] font-medium text-sidebar-foreground hover:bg-sidebar-row-hover"
              >
                {editing === folder.id ? (
                  <input
                    autoFocus
                    defaultValue={folder.name}
                    onBlur={(event) => {
                      const name = event.currentTarget.value.trim();
                      if (name) patchFolder(folder.id, { name });
                      setEditing(null);
                    }}
                    onKeyDown={(event) => {
                      if (event.key === "Enter") event.currentTarget.blur();
                      if (event.key === "Escape") setEditing(null);
                    }}
                    className="min-w-0 flex-1 rounded bg-transparent px-1 outline-1 outline-ring"
                  />
                ) : (
                  <button
                    type="button"
                    aria-label={folder.collapsed ? gt("Expand folder") : gt("Collapse folder")}
                    onClick={() => patchFolder(folder.id, { collapsed: !folder.collapsed })}
                    className="flex min-w-0 items-center gap-1.5 text-left"
                  >
                    <span className="truncate">{folder.name}</span>
                    {folder.collapsed ? (
                      <ChevronRightIcon className="size-3.5 shrink-0 text-sidebar-muted-foreground" />
                    ) : (
                      <ChevronDownIcon className="size-3.5 shrink-0 text-sidebar-muted-foreground" />
                    )}
                  </button>
                )}
                <span className="ms-auto flex items-center gap-1.5 text-xs font-normal text-sidebar-muted-foreground tabular-nums">
                  {folder.collapsed && working ? <StatusMark status="working" /> : null}
                  {inFolder.length}
                </span>
              </div>
              {folder.collapsed ? null : (
                <ul className="flex flex-col gap-px">{inFolder.map(row)}</ul>
              )}
            </section>
          );
        })}
        <ul
          className="flex flex-col gap-px"
          onDragOver={(event) => event.preventDefault()}
          onDrop={() => {
            const dragged = drag.current;
            if (dragged?.kind === "chat") assign(dragged.id, null);
            drag.current = null;
          }}
        >
          {loose.map(row)}
        </ul>
      </SidebarContent>
      <Menu open={menu !== null} onOpenChange={(open) => (open ? undefined : setMenu(null))}>
        {menu ? (
          <MenuPopup
            align="start"
            side="bottom"
            sideOffset={0}
            className="min-w-48"
            anchor={{
              getBoundingClientRect: () => new DOMRect(menu.x, menu.y, 0, 0),
            }}
          >
            {menu.kind === "thread" ? (
              <>
                <MenuSub>
                  <MenuSubTrigger>{gt("Move to folder")}</MenuSubTrigger>
                  <MenuSubPopup>
                    <MenuItem onClick={() => assign(threadKey(menu.thread), null)}>
                      {gt("No folder")}
                    </MenuItem>
                    {state.folders.map((folder) => (
                      <MenuItem
                        key={folder.id}
                        onClick={() => assign(threadKey(menu.thread), folder.id)}
                      >
                        {folder.name}
                      </MenuItem>
                    ))}
                    {state.folders.length > 0 ? <MenuSeparator /> : null}
                    <MenuItem onClick={() => addFolder(menu.thread)}>
                      <PlusIcon />
                      {gt("New folder")}
                    </MenuItem>
                  </MenuSubPopup>
                </MenuSub>
                <MenuSeparator />
                <MenuItem onClick={() => void archiveChat(menu.thread)}>{gt("Archive")}</MenuItem>
                <MenuItem
                  className="text-destructive-foreground"
                  onClick={() => void deleteChat(menu.thread)}
                >
                  {gt("Delete")}
                </MenuItem>
              </>
            ) : (
              <>
                <MenuItem onClick={() => setEditing(menu.folderId)}>{gt("Rename folder")}</MenuItem>
                <MenuItem
                  className="text-destructive-foreground"
                  onClick={() => removeFolder(menu.folderId)}
                >
                  {gt("Delete folder")}
                </MenuItem>
              </>
            )}
          </MenuPopup>
        ) : null}
      </Menu>
      <SidebarChromeFooter />
    </>
  );
}
