// CC3: the small forms of the GitHub panel (new repository, new branch, link folder, clone, delete).
import { useGT } from "gt-react";
import { useState } from "react";

import { Button } from "~/components/ui/button";
import { Checkbox } from "~/components/ui/checkbox";
import { Input } from "~/components/ui/input";

import { Field, FormCard, SimpleSelect } from "./GitHubUi";

export function NewRepoForm(props: {
  readonly folderFree: boolean;
  readonly parent: string;
  readonly busy: boolean;
  readonly onSubmit: (input: {
    name: string;
    description: string;
    visibility: "private" | "public";
    inFolder: boolean;
  }) => void;
  readonly onCancel: () => void;
}) {
  const gt = useGT();
  const [name, setName] = useState("");
  const [description, setDescription] = useState("");
  const [visibility, setVisibility] = useState<"private" | "public">("private");
  const [where, setWhere] = useState(props.folderFree ? "folder" : "new");
  return (
    <FormCard
      title={gt("New repository")}
      submitLabel={gt("Create repository")}
      busy={props.busy}
      disabled={!name.trim()}
      onCancel={props.onCancel}
      onSubmit={() =>
        props.onSubmit({
          name: name.trim(),
          description,
          visibility,
          inFolder: where === "folder",
        })
      }
    >
      <Field label={gt("Name")}>
        <Input
          size="compact"
          autoFocus
          value={name}
          spellCheck={false}
          onChange={(e) => setName(e.target.value)}
        />
      </Field>
      <Field label={gt("Description (optional)")}>
        <Input
          size="compact"
          value={description}
          onChange={(e) => setDescription(e.target.value)}
        />
      </Field>
      <div className="grid grid-cols-2 gap-2">
        <Field label={gt("Visibility")}>
          <SimpleSelect
            ariaLabel={gt("Visibility")}
            value={visibility}
            onChange={(value) => setVisibility(value as "private" | "public")}
            items={[
              { value: "private", label: gt("Private") },
              { value: "public", label: gt("Public") },
            ]}
          />
        </Field>
        <Field label={gt("Location")}>
          <SimpleSelect
            ariaLabel={gt("Location")}
            value={where}
            onChange={setWhere}
            items={[
              ...(props.folderFree ? [{ value: "folder", label: gt("This chat's folder") }] : []),
              { value: "new", label: gt("New folder in {parent}", { parent: props.parent }) },
            ]}
          />
        </Field>
      </div>
    </FormCard>
  );
}

export function NewBranchForm(props: {
  readonly base: string;
  readonly local: boolean;
  readonly busy: boolean;
  readonly onSubmit: (input: { name: string; publish: boolean }) => void;
  readonly onCancel: () => void;
}) {
  const gt = useGT();
  const [name, setName] = useState("");
  const [publish, setPublish] = useState(true);
  return (
    <FormCard
      title={gt("New branch from {base}", { base: props.base })}
      submitLabel={gt("Create branch")}
      busy={props.busy}
      disabled={!name.trim()}
      onCancel={props.onCancel}
      onSubmit={() => props.onSubmit({ name: name.trim(), publish })}
    >
      <Field label={gt("Branch name")}>
        <Input
          size="compact"
          autoFocus
          value={name}
          spellCheck={false}
          onChange={(e) => setName(e.target.value)}
        />
      </Field>
      {props.local ? (
        <label className="flex items-center gap-2 text-xs text-muted-foreground">
          <Checkbox checked={publish} onCheckedChange={(checked) => setPublish(checked === true)} />
          {gt("Publish to GitHub")}
        </label>
      ) : (
        <p className="text-xs text-muted-foreground">
          {gt("The branch is created on GitHub. Clone the repository to work on it.")}
        </p>
      )}
    </FormCard>
  );
}

export function LinkFolderForm(props: {
  readonly initial: string;
  readonly chatFolder: string | null;
  readonly onSubmit: (folder: string, useInChat: boolean) => void;
  readonly onCancel: () => void;
}) {
  const gt = useGT();
  const [folder, setFolder] = useState(props.initial);
  return (
    <FormCard
      title={gt("Link a folder")}
      submitLabel={gt("Link folder")}
      disabled={!folder.trim()}
      onCancel={props.onCancel}
      onSubmit={() => props.onSubmit(folder.trim(), false)}
      extraAction={
        <Button
          type="button"
          size="sm"
          variant="outline"
          disabled={!folder.trim()}
          onClick={() => props.onSubmit(folder.trim(), true)}
        >
          {gt("Link and use in this chat")}
        </Button>
      }
    >
      <Field label={gt("Folder")}>
        <Input
          size="compact"
          autoFocus
          value={folder}
          spellCheck={false}
          onChange={(e) => setFolder(e.target.value)}
        />
      </Field>
      {props.chatFolder ? (
        <button
          type="button"
          className="self-start text-xs text-muted-foreground underline-offset-2 hover:text-foreground hover:underline"
          onClick={() => setFolder(props.chatFolder ?? "")}
        >
          {gt("Use this chat's folder")}
        </button>
      ) : null}
    </FormCard>
  );
}

export function CloneForm(props: {
  readonly repo: string;
  readonly initial: string;
  readonly onSubmit: (target: string) => void;
  readonly onCancel: () => void;
}) {
  const gt = useGT();
  const [target, setTarget] = useState(props.initial);
  return (
    <FormCard
      title={gt("Clone {repo}", { repo: props.repo })}
      submitLabel={gt("Clone")}
      disabled={!target.trim()}
      onCancel={props.onCancel}
      onSubmit={() => props.onSubmit(target.trim())}
    >
      <Field label={gt("Clone into")}>
        <Input
          size="compact"
          autoFocus
          value={target}
          spellCheck={false}
          onChange={(e) => setTarget(e.target.value)}
        />
      </Field>
      <p className="text-xs text-muted-foreground">
        {gt("The repository becomes a CC3 project and its folder is linked.")}
      </p>
    </FormCard>
  );
}

export function DeleteForm(props: {
  readonly repo: string;
  readonly busy: boolean;
  readonly onSubmit: () => void;
  readonly onCancel: () => void;
}) {
  const gt = useGT();
  const [typed, setTyped] = useState("");
  return (
    <FormCard
      destructive
      title={gt("Delete {repo}", { repo: props.repo })}
      submitLabel={gt("Delete repository")}
      busy={props.busy}
      disabled={typed !== props.repo}
      onCancel={props.onCancel}
      onSubmit={props.onSubmit}
    >
      <p className="text-xs text-muted-foreground">
        {gt("This deletes the repository on GitHub for good. Type {repo} to confirm.", {
          repo: props.repo,
        })}
      </p>
      <Input
        size="compact"
        autoFocus
        value={typed}
        spellCheck={false}
        aria-label={gt("Repository name")}
        onChange={(e) => setTyped(e.target.value)}
      />
    </FormCard>
  );
}
