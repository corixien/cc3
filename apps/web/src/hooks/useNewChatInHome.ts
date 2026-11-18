// CC3: the sidebar's "+" starts a chat in the user's home directory, creating a "Home" project for
// it on first use. Returns null where the home directory is unknown (non-Electron clients).
import { RegistryContext } from "@effect/atom-react";
import { scopeProjectRef } from "@t3tools/client-runtime/environment";
import { thrownErrorMessage } from "@t3tools/client-runtime/errors";
import {
  isAtomCommandInterrupted,
  runAtomCommand,
  squashAtomCommandFailure,
} from "@t3tools/client-runtime/state/runtime";
import { findProjectByPath } from "@t3tools/client-runtime/state/projects";
import { msg } from "gt-react";
import { useContext } from "react";

import { stackedThreadToast, toastManager } from "~/components/ui/toast";
import { m } from "~/i18n/messages";
import { newProjectId } from "~/lib/utils";
import { usePrimaryEnvironmentId } from "~/state/environments";
import { useProjects } from "~/state/entities";
import { projectEnvironment } from "~/state/projects";
import { useNewThreadHandler } from "./useHandleNewThread";

export function useNewChatInHome(): (() => Promise<void>) | null {
  const registry = useContext(RegistryContext);
  const projects = useProjects();
  const environmentId = usePrimaryEnvironmentId();
  const handleNewThread = useNewThreadHandler();
  const home = typeof window === "undefined" ? undefined : window.cc3HomeDir;
  if (!home || environmentId === null) return null;

  return async () => {
    const project = findProjectByPath(
      projects.filter((candidate) => candidate.environmentId === environmentId),
      home,
    );
    let projectId = project?.id ?? null;
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
            title: m(msg("Home")),
            workspaceRoot: home,
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
              title: m(msg("Could not start a chat")),
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
