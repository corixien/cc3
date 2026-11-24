// CC3: a location bar that always shows the current folder and accepts a typed or pasted path.
import { useGT } from "gt-react";

export function PathLocationField(props: {
  readonly directory: string;
  readonly onSubmit: (path: string) => void;
}) {
  const gt = useGT();
  return (
    <input
      // A new folder replaces the text; typing in between is kept until then.
      key={props.directory}
      defaultValue={props.directory}
      spellCheck={false}
      aria-label={gt("File path")}
      className="h-6 w-full min-w-0 rounded-sm bg-transparent px-1.5 text-xs text-foreground outline-none hover:bg-accent/40 focus:bg-accent/40 focus:ring-1 focus:ring-ring"
      onFocus={(event) => event.currentTarget.select()}
      onKeyDown={(event) => {
        const input = event.currentTarget;
        if (event.key === "Enter") {
          const value = input.value.trim();
          if (value && value !== props.directory) props.onSubmit(value);
          else input.value = props.directory;
        } else if (event.key === "Escape") {
          input.value = props.directory;
          input.blur();
        }
      }}
      onBlur={(event) => {
        event.currentTarget.value = props.directory;
      }}
    />
  );
}
