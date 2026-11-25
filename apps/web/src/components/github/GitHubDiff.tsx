// CC3: renders a unified patch like the Diff panel does.
import { FileDiff } from "@pierre/diffs/react";
import { useGT } from "gt-react";

import { useRenderedAppearance } from "~/hooks/useTheme";
import { getRenderablePatch, resolveDiffThemeName } from "~/lib/diffRendering";
import { PREFERRED_HIGHLIGHTER } from "~/lib/syntaxHighlighting";

import { DiffWorkerPoolProvider } from "../DiffWorkerPoolProvider";
import { DIFF_VIEW_UNSAFE_CSS } from "../diffs/StyledDiffCodeView";

export function PatchFiles(props: { readonly patch: string; readonly scope: string }) {
  const gt = useGT();
  const appearance = useRenderedAppearance();
  const renderable = getRenderablePatch(props.patch, props.scope);
  const options = {
    diffStyle: "unified" as const,
    overflow: "scroll" as const,
    theme: resolveDiffThemeName(appearance),
    themeType: appearance,
    preferredHighlighter: PREFERRED_HIGHLIGHTER,
    unsafeCSS: DIFF_VIEW_UNSAFE_CSS,
  };
  if (renderable?.kind === "files") {
    return (
      <DiffWorkerPoolProvider>
        <div className="flex flex-col gap-3 p-2">
          {renderable.files.map((file) => (
            <div
              key={file.cacheKey ?? file.name}
              className="overflow-hidden rounded-md border border-border/60"
            >
              <FileDiff fileDiff={file} options={options} />
            </div>
          ))}
        </div>
      </DiffWorkerPoolProvider>
    );
  }
  if (renderable?.kind === "raw") {
    return <pre className="overflow-auto p-3 font-mono text-[11px]">{renderable.text}</pre>;
  }
  return <div className="p-3 text-xs text-muted-foreground">{gt("No file changes.")}</div>;
}
