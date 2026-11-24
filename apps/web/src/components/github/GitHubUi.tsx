// CC3: building blocks that keep every form of the GitHub panel looking the same.
import { useGT } from "gt-react";
import type { ReactNode } from "react";

import { Button } from "~/components/ui/button";
import {
  Select,
  SelectItem,
  SelectPopup,
  SelectTrigger,
  SelectValue,
} from "~/components/ui/select";
import { Spinner } from "~/components/ui/spinner";

export function Field(props: { readonly label: string; readonly children: ReactNode }) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-[11px] font-medium text-muted-foreground">{props.label}</span>
      {props.children}
    </label>
  );
}

/** One card per task: title, labelled fields, Cancel and one primary action. */
export function FormCard(props: {
  readonly title: string;
  readonly children: ReactNode;
  readonly submitLabel: string;
  readonly onSubmit: () => void;
  readonly onCancel: () => void;
  readonly busy?: boolean;
  readonly disabled?: boolean;
  readonly destructive?: boolean;
  readonly extraAction?: ReactNode;
}) {
  const gt = useGT();
  return (
    <form
      className="m-3 flex flex-col gap-3 rounded-lg border border-border/70 bg-card p-3"
      onSubmit={(event) => {
        event.preventDefault();
        if (!props.busy && !props.disabled) props.onSubmit();
      }}
      onKeyDown={(event) => {
        if (event.key === "Escape") props.onCancel();
      }}
    >
      <div className="text-sm font-medium text-foreground">{props.title}</div>
      {props.children}
      <div className="flex items-center justify-end gap-2">
        {props.extraAction}
        <Button type="button" size="sm" variant="ghost" onClick={props.onCancel}>
          {gt("Cancel")}
        </Button>
        <Button
          type="submit"
          size="sm"
          variant={props.destructive ? "destructive" : "default"}
          disabled={props.busy || props.disabled}
        >
          {props.busy ? <Spinner className="size-3.5" /> : props.submitLabel}
        </Button>
      </div>
    </form>
  );
}

export function SimpleSelect(props: {
  readonly value: string;
  readonly onChange: (value: string) => void;
  readonly items: ReadonlyArray<{ value: string; label: string }>;
  readonly ariaLabel: string;
  readonly icon?: ReactNode;
  readonly className?: string;
}) {
  return (
    <Select
      value={props.value}
      onValueChange={(value) => {
        if (value) props.onChange(value);
      }}
      items={[...props.items]}
    >
      <SelectTrigger size="sm" aria-label={props.ariaLabel} className={props.className}>
        {props.icon}
        <SelectValue />
      </SelectTrigger>
      <SelectPopup>
        {props.items.map((item) => (
          <SelectItem key={item.value} value={item.value}>
            {item.label}
          </SelectItem>
        ))}
      </SelectPopup>
    </Select>
  );
}
