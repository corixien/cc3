// CC3: the status strip's folder opens the system folder dialog and starts a chat in the pick,
// creating a project for the folder on first use.
import { RegistryContext } from "@effect/atom-react";
import { scopeProjectRef } from "@t3tools/client-runtime/environment";
import { thrownErrorMessage } from "@t3tools/client-runtime/errors";
import {
  isAtomCommandInterrupted,
  runAtomCommand,
  squashAtomCommandFailure,
} from "@t3tools/client-runtime/state/runtime";
import {
  findProjectByPath,
  inferProjectTitleFromPath,
} from "@t3tools/client-runtime/state/projects";
import type { EnvironmentId } from "@t3tools/contracts";
import { msg } from "gt-react";
import { useContext } from "react";

import { stackedThreadToast, toastManager } from "~/components/ui/toast";
import { m } from "~/i18n/messages";
import { newProjectId } from "~/lib/utils";
import { readLocalApi } from "~/localApi";
import { useProjects } from "~/state/entities";
import { projectEnvironment } from "~/state/projects";
import { useNewThreadHandler } from "./useHandleNewThread";

export function useOpenFolderChat(environmentId: EnvironmentId | null): () => Promise<void> {
  const registry = useContext(RegistryContext);
  const projects = useProjects();
  const handleNewThread = useNewThreadHandler();

  return async () => {
    const api = readLocalApi();
    if (!api || environmentId === null) return;
    const picked = await api.dialogs.pickFolder().catch(() => null);
    if (!picked) return;
    const existing = findProjectByPath(
      projects.filter((candidate) => candidate.environmentId === environmentId),
      picked,
    );
    let projectId = existing?.id ?? null;
    if (projectId === null) {
      const id = newProjectId();
      const command = projectEnvironment.create;
      const result = await runAtomCommand(
        registry,
        command,
        {
          environmentId,
          input: {
            projectId: id,
            title: inferProjectTitleFromPath(picked),
            workspaceRoot: picked,
            createWorkspaceRootIfMissing: false,
            defaultModelSelection: null,
          },
        },
        { label: command.label, reportFailure: false, reportDefect: true },
      );
      if (result._tag !== "Success") {
        if (!isAtomCommandInterrupted(result)) {
          toastManager.add(
            stackedThreadToast({
              type: "error",
              title: m(msg("Could not open folder")),
              description:
                thrownErrorMessage(squashAtomCommandFailure(result), m) ??
                m(msg("An error occurred.")),
            }),
          );
        }
        return;
      }
      projectId = id;
    }
    await handleNewThread(scopeProjectRef(environmentId, projectId));
  };
}
