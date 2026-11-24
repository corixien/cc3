// CC3: like a file manager's location bar, a click on the empty part of the breadcrumbs shows the full path as selectable text.
import { useGT } from "gt-react";
import { type ReactNode, useState } from "react";

export function BreadcrumbPathField(props: {
  readonly path: string | null;
  readonly children: ReactNode;
}) {
  const gt = useGT();
  const [editing, setEditing] = useState(false);
  if (editing && props.path) {
    return (
      <input
        readOnly
        autoFocus
        aria-label={gt("File path")}
        value={props.path}
        className="h-6 w-full min-w-0 rounded-sm bg-accent/40 px-1.5 text-xs text-foreground outline-none ring-1 ring-ring"
        onFocus={(event) => event.currentTarget.select()}
        onBlur={() => setEditing(false)}
        onKeyDown={(event) => {
          if (event.key === "Escape" || event.key === "Enter") setEditing(false);
        }}
      />
    );
  }
  return (
    <div
      className="flex h-full w-max min-w-full items-center text-xs"
      onClick={(event) => {
        if (event.target === event.currentTarget && props.path) setEditing(true);
      }}
    >
      {props.children}
    </div>
  );
}
