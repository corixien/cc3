// CC3: the thread sidebar. A flat list of chats (title and a live status only) that can be sorted
// into collapsible folders. Replaces the project-grouped upstream sidebar in the app layout.
import { scopeThreadRef } from "@t3tools/client-runtime/environment";
import {
  threadRuntimeIsActive,
  type EnvironmentThreadShell,
} from "@t3tools/client-runtime/state/shell";
import { isAtomCommandInterrupted, settlePromise } from "@t3tools/client-runtime/state/runtime";
import { useNavigate, useParams } from "@tanstack/react-router";
import * as Schema from "effect/Schema";
import { useGT } from "gt-react";
import { ChevronDownIcon, ChevronRightIcon, FolderPlusIcon, PlusIcon } from "lucide-react";
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
import { SidebarContent } from "./ui/sidebar";
import { Spinner } from "./ui/spinner";

const FolderState = Schema.Struct({
  folders: Schema.Array(
    Schema.Struct({ id: Schema.String, name: Schema.String, collapsed: Schema.Boolean }),
  ),
  assignments: Schema.Record(Schema.String, Schema.String),
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

export default function Cc3Sidebar() {
  const gt = useGT();
  const navigate = useNavigate();
  const params = useParams({ strict: false }) as { environmentId?: string; threadId?: string };
  const shells = useThreadShells();
  const { archiveThread, deleteThread } = useThreadActions();
  const { scratchEnvironmentId, startScratchThread } = useScratchProject();
  const [state, setState] = useLocalStorage(STORAGE_KEY, EMPTY_FOLDERS, FolderState);
  const [editing, setEditing] = useState<string | null>(null);
  const dragKey = useRef<string | null>(null);

  const threads = useMemo(
    () =>
      shells
        .filter((thread) => thread.archivedAt === null && thread.deletedAt === null)
        .toSorted((a, b) => activityOf(b) - activityOf(a)),
    [shells],
  );
  const folderIds = new Set(state.folders.map((folder) => folder.id));
  const folderOf = (thread: EnvironmentThreadShell) => {
    const id = state.assignments[threadKey(thread)];
    return id !== undefined && folderIds.has(id) ? id : null;
  };
  const loose = threads.filter((thread) => folderOf(thread) === null);

  const assign = (key: string, folderId: string | null) =>
    setState((current) => {
      const assignments = { ...current.assignments };
      if (folderId === null) delete assignments[key];
      else assignments[key] = folderId;
      return { ...current, assignments };
    });
  const addFolder = () => {
    const id = crypto.randomUUID();
    setState((current) => ({
      ...current,
      folders: [...current.folders, { id, name: gt("New folder"), collapsed: false }],
    }));
    setEditing(id);
  };
  const patchFolder = (id: string, patch: Partial<FolderState["folders"][number]>) =>
    setState((current) => ({
      ...current,
      folders: current.folders.map((folder) => (folder.id === id ? { ...folder, ...patch } : folder)),
    }));
  const removeFolder = (id: string) =>
    setState((current) => ({
      folders: current.folders.filter((folder) => folder.id !== id),
      assignments: Object.fromEntries(
        Object.entries(current.assignments).filter(([, folderId]) => folderId !== id),
      ),
    }));

  const open = (thread: EnvironmentThreadShell) =>
    void navigate({
      to: "/$environmentId/$threadId",
      params: buildThreadRouteParams(scopeThreadRef(thread.environmentId, thread.id)),
    });
  const homeEnvironmentId = scratchEnvironmentId(null);
  const newChat = () => {
    if (homeEnvironmentId !== null) void startScratchThread(homeEnvironmentId);
  };

  const threadMenu = (thread: EnvironmentThreadShell, position: { x: number; y: number }) => {
    const api = readLocalApi();
    if (!api) return;
    void (async () => {
      const clicked = await settlePromise(() =>
        api.contextMenu.show(
          [
            {
              id: "move",
              label: gt("Move to folder"),
              children: [
                { id: "move:none", label: gt("No folder") },
                ...state.folders.map((folder) => ({
                  id: `move:${folder.id}`,
                  label: folder.name,
                })),
              ],
            },
            { id: "archive", label: gt("Archive") },
            { id: "delete", label: gt("Delete"), destructive: true },
          ],
          position,
        ),
      );
      if (clicked._tag === "Failure" || clicked.value === null) return;
      const id = String(clicked.value);
      const ref = scopeThreadRef(thread.environmentId, thread.id);
      if (id.startsWith("move:")) {
        const target = id.slice("move:".length);
        assign(threadKey(thread), target === "none" ? null : target);
      } else if (id === "archive") {
        const result = await archiveThread(ref, {});
        if (result._tag === "Failure" && !isAtomCommandInterrupted(result)) return;
      } else if (id === "delete") {
        const confirmed = await settlePromise(() =>
          api.dialogs.confirm(gt('Delete chat "{title}"?', { title: thread.title }), {
            variant: "destructive",
          }),
        );
        if (confirmed._tag === "Failure" || !confirmed.value) return;
        await deleteThread(ref);
      }
    })();
  };

  const folderMenu = (folderId: string, position: { x: number; y: number }) => {
    const api = readLocalApi();
    if (!api) return;
    void (async () => {
      const clicked = await settlePromise(() =>
        api.contextMenu.show(
          [
            { id: "rename", label: gt("Rename folder") },
            { id: "delete", label: gt("Delete folder"), destructive: true },
          ],
          position,
        ),
      );
      if (clicked._tag === "Failure") return;
      if (clicked.value === "rename") setEditing(folderId);
      else if (clicked.value === "delete") removeFolder(folderId);
    })();
  };

  const row = (thread: EnvironmentThreadShell) => {
    const selected = params.environmentId === thread.environmentId && params.threadId === thread.id;
    const status = statusOf(thread);
    return (
      <li key={threadKey(thread)}>
        <button
          type="button"
          draggable
          onDragStart={() => (dragKey.current = threadKey(thread))}
          onClick={() => open(thread)}
          onContextMenu={(event) => {
            event.preventDefault();
            threadMenu(thread, { x: event.clientX, y: event.clientY });
          }}
          className={cn(
            "flex h-8 w-full items-center gap-2 rounded-md px-2.5 text-left text-sm text-sidebar-foreground hover:bg-sidebar-row-hover",
            selected && "bg-sidebar-row-selected",
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
      <SidebarContent className="min-h-0 gap-1 px-2 pt-2">
        <div className="flex items-center gap-1 pb-1">
          <button
            type="button"
            onClick={newChat}
            disabled={homeEnvironmentId === null}
            className="flex h-8 flex-1 items-center gap-2 rounded-md px-2.5 text-sm font-medium text-sidebar-foreground hover:bg-sidebar-row-hover disabled:opacity-50"
          >
            <PlusIcon className="size-4" />
            {gt("New chat")}
          </button>
          <button
            type="button"
            aria-label={gt("New folder")}
            onClick={addFolder}
            className="flex size-8 items-center justify-center rounded-md text-sidebar-muted-foreground hover:bg-sidebar-row-hover hover:text-sidebar-foreground"
          >
            <FolderPlusIcon className="size-4" />
          </button>
        </div>
        {state.folders.map((folder) => {
          const inFolder = threads.filter((thread) => folderOf(thread) === folder.id);
          const working = inFolder.some((thread) => statusOf(thread) !== null);
          return (
            <section
              key={folder.id}
              onDragOver={(event) => event.preventDefault()}
              onDrop={() => {
                if (dragKey.current) assign(dragKey.current, folder.id);
                dragKey.current = null;
              }}
            >
              <div
                onContextMenu={(event) => {
                  event.preventDefault();
                  folderMenu(folder.id, { x: event.clientX, y: event.clientY });
                }}
                className="flex h-7 items-center gap-1 rounded-md px-1.5 text-xs font-medium text-sidebar-muted-foreground hover:bg-sidebar-row-hover"
              >
                <button
                  type="button"
                  aria-label={folder.collapsed ? gt("Expand folder") : gt("Collapse folder")}
                  onClick={() => patchFolder(folder.id, { collapsed: !folder.collapsed })}
                  className="flex min-w-0 flex-1 items-center gap-1 text-left"
                >
                  {folder.collapsed ? (
                    <ChevronRightIcon className="size-3.5 shrink-0" />
                  ) : (
                    <ChevronDownIcon className="size-3.5 shrink-0" />
                  )}
                  {editing === folder.id ? (
                    <input
                      autoFocus
                      defaultValue={folder.name}
                      onClick={(event) => event.stopPropagation()}
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
                    <span className="truncate">{folder.name}</span>
                  )}
                </button>
                {folder.collapsed && working ? <StatusMark status="working" /> : null}
                <span className="tabular-nums">{inFolder.length}</span>
              </div>
              {folder.collapsed ? null : <ul className="flex flex-col">{inFolder.map(row)}</ul>}
            </section>
          );
        })}
        <ul
          className="flex flex-col"
          onDragOver={(event) => event.preventDefault()}
          onDrop={() => {
            if (dragKey.current) assign(dragKey.current, null);
            dragKey.current = null;
          }}
        >
          {loose.map(row)}
        </ul>
      </SidebarContent>
      <SidebarChromeFooter />
    </>
  );
}
