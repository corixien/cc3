// CC3: a new chat is a real server thread from the moment "+" is pressed, so its provider session
// can start before the first message. An empty chat that is left unused is deleted again.
import { runAtomCommand } from "@t3tools/client-runtime/state/runtime";
import type { EnvironmentId, ModelSelection, ScopedProjectRef, ThreadId } from "@t3tools/contracts";
import type { AnyRouter } from "@tanstack/react-router";
import type { AtomRegistry } from "effect/reactivity";

import { useComposerDraftStore, type DraftId } from "./composerDraftStore";
import { resolveProjectDefaultProviderModelSelection } from "./providerInstances";
import { appAtomRegistry } from "./rpc/atomRegistry";
import { environmentServerConfigsAtom } from "./state/server";
import { threadEnvironment } from "./state/threads";
import { useSplitViewStore } from "./splitViewStore";
import { resolveThreadRouteTarget } from "./threadRoutes";
import { DEFAULT_SERVER_SETTINGS } from "@t3tools/contracts";

const STORAGE_KEY = "cc3:unused-new-chats:v1";
const GRACE_MS = 1_500;

interface TrackedChat {
  readonly environmentId: EnvironmentId;
  readonly threadId: ThreadId;
}

const readTracked = (): TrackedChat[] => {
  try {
    const parsed: unknown = JSON.parse(globalThis.localStorage?.getItem(STORAGE_KEY) ?? "[]");
    return Array.isArray(parsed) ? (parsed as TrackedChat[]) : [];
  } catch {
    return [];
  }
};

const writeTracked = (chats: ReadonlyArray<TrackedChat>) => {
  try {
    globalThis.localStorage?.setItem(STORAGE_KEY, JSON.stringify(chats));
  } catch {
    // storage unavailable: the chat is then only cleaned up while this window stays open
  }
};

export function trackNewChat(chat: TrackedChat): void {
  const rest = readTracked().filter((entry) => entry.threadId !== chat.threadId);
  writeTracked([...rest, chat]);
}

const untrack = (threadId: ThreadId) =>
  writeTracked(readTracked().filter((entry) => entry.threadId !== threadId));

/** The model the draft shows: its own pick, else the project's default provider and model. */
function resolveDraftModelSelection(
  draftId: DraftId,
  projectRef: ScopedProjectRef,
): ModelSelection | null {
  const composer = useComposerDraftStore.getState().getComposerDraft(draftId);
  const active = composer?.activeProvider ?? useComposerDraftStore.getState().stickyActiveProvider;
  const picked = active ? (composer?.modelSelectionByProvider[active] ?? null) : null;
  if (picked) return picked;
  const config = appAtomRegistry.get(environmentServerConfigsAtom).get(projectRef.environmentId);
  return resolveProjectDefaultProviderModelSelection(
    config?.providers ?? [],
    config?.settings ?? DEFAULT_SERVER_SETTINGS,
    projectRef.projectId,
  );
}

/**
 * Creates the server thread for a fresh draft. The draft route hands over to the thread route
 * by itself once the thread exists.
 */
export async function materializeNewChat(
  registry: AtomRegistry.AtomRegistry,
  input: { readonly draftId: DraftId; readonly projectRef: ScopedProjectRef },
): Promise<void> {
  const store = useComposerDraftStore.getState();
  const session = store.getDraftSession(input.draftId);
  const modelSelection = resolveDraftModelSelection(input.draftId, input.projectRef);
  if (!session || !modelSelection) return;
  const result = await runAtomCommand(
    registry,
    threadEnvironment.create,
    {
      environmentId: input.projectRef.environmentId,
      input: {
        threadId: session.threadId,
        projectId: input.projectRef.projectId,
        // oxlint-disable-next-line t3code/no-untranslated-copy -- replaced by the title generated from the first message
        title: "New chat",
        modelSelection,
        runtimeMode: session.runtimeMode,
        interactionMode: session.interactionMode,
        branch: session.branch,
        worktreePath: session.worktreePath,
        createdAt: new Date().toISOString(),
      },
    },
    { label: threadEnvironment.create.label, reportFailure: false, reportDefect: false },
  );
  if (result._tag === "Success") {
    trackNewChat({ environmentId: input.projectRef.environmentId, threadId: session.threadId });
  }
}

/** Deletes tracked chats that are neither on screen nor have received a message. */
export function startNewChatCleanup(registry: AtomRegistry.AtomRegistry, router: AnyRouter) {
  const shownThreadIds = (): Set<ThreadId> => {
    const shown = new Set<ThreadId>();
    const params = router.state.matches[router.state.matches.length - 1]?.params ?? {};
    const target = resolveThreadRouteTarget(params);
    if (target?.kind === "server") shown.add(target.threadRef.threadId);
    if (target?.kind === "draft") {
      const draft = useComposerDraftStore.getState().getDraftSession(target.draftId);
      if (draft) shown.add(draft.threadId);
    }
    const secondary = useSplitViewStore.getState().secondary;
    if (secondary) shown.add(secondary.threadId);
    return shown;
  };

  const sweep = async () => {
    const shown = shownThreadIds();
    for (const chat of readTracked()) {
      if (shown.has(chat.threadId)) continue;
      untrack(chat.threadId);
      // The server deletes only a thread with no runs and no messages.
      await runAtomCommand(
        registry,
        threadEnvironment.delete,
        {
          environmentId: chat.environmentId,
          input: { threadId: chat.threadId, onlyIfUnused: true },
        },
        { label: threadEnvironment.delete.label, reportFailure: false, reportDefect: false },
      ).catch(() => undefined);
    }
  };

  let timer: ReturnType<typeof setTimeout> | undefined;
  const schedule = (delay: number) => {
    if (timer !== undefined) clearTimeout(timer);
    timer = setTimeout(() => void sweep(), delay);
  };
  const unsubscribe = router.subscribe("onResolved", () => schedule(GRACE_MS));
  // Chats left behind by a closed window or a crash.
  schedule(8_000);
  return () => {
    unsubscribe();
    if (timer !== undefined) clearTimeout(timer);
  };
}
