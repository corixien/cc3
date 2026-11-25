// @effect-diagnostics nodeBuiltinImport:off -- A fixed set of short-lived gh and git calls inside one IPC handler; no Effect runtime is involved.
// CC3: backend of the GitHub panel. Runs a fixed set of read operations through the user's own `gh` and `git`,
// with every argument validated; nothing here takes a free-form command.
import * as Electron from "electron";
import { execFile, spawn } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  existsSync,
  readdirSync,
  mkdtempSync,
  openSync,
  readFileSync,
  realpathSync,
  rmSync,
  statSync,
  writeFileSync,
} from "node:fs";
import { tmpdir } from "node:os";
import { dirname, join, resolve, sep } from "node:path";

export const CC3_GITHUB_CHANNEL = "cc3:github";

const REPO = /^[\w.-]+\/[\w.-]+$/;
const OWNER = /^[\w.-]+$/;
const REF = /^[\w./@+-]+$/;

type Failure = { readonly ok: false; readonly error: string };
type Success = { readonly ok: true; readonly data: unknown };

const fail = (error: string): Failure => ({ ok: false, error });

function run(
  command: "gh" | "git" | "tar",
  args: readonly string[],
  options: { cwd?: string; maxBuffer?: number; timeout?: number; okExit?: readonly number[] } = {},
): Promise<{ stdout: string } | Failure> {
  return new Promise((resolve) => {
    execFile(
      command,
      [...args],
      {
        cwd: options.cwd,
        maxBuffer: options.maxBuffer ?? 8 * 1024 * 1024,
        timeout: options.timeout ?? 45_000,
        env: { ...process.env, GH_PROMPT_DISABLED: "1", GIT_TERMINAL_PROMPT: "0", NO_COLOR: "1" },
      },
      (error, stdout, stderr) => {
        if (error && typeof error.code === "number" && options.okExit?.includes(error.code)) {
          resolve({ stdout });
          return;
        }
        if (error)
          resolve(fail((stderr || error.message).trim().split("\n").slice(0, 4).join("\n")));
        else resolve({ stdout });
      },
    );
  });
}

const isFailure = (value: object): value is Failure => "ok" in value;

const safeRef = (value: unknown): value is string =>
  typeof value === "string" && REF.test(value) && !value.startsWith("-") && !value.includes("..");
const safeRepo = (value: unknown): value is string => typeof value === "string" && REPO.test(value);
const safePath = (value: unknown): value is string =>
  typeof value === "string" &&
  !value.startsWith("-") &&
  !value.split("/").includes("..") &&
  !value.includes("\0");
const safeDir = (value: unknown): value is string => {
  if (typeof value !== "string" || !value.startsWith("/")) return false;
  try {
    return statSync(value).isDirectory();
  } catch {
    return false;
  }
};

const encodePath = (path: string) =>
  path.split("/").filter(Boolean).map(encodeURIComponent).join("/");
const contentsUrl = (repo: string, path: string, ref: string) =>
  `repos/${repo}/contents/${encodePath(path)}?ref=${encodeURIComponent(ref)}`;

function repoFromRemote(url: string): string | null {
  const match = /github\.com[:/]([\w.-]+)\/([\w.-]+?)(?:\.git)?\/?$/.exec(url.trim());
  return match ? `${match[1]}/${match[2]}` : null;
}

interface RemoteFile {
  readonly path: string;
  readonly rel: string;
  readonly size: number;
  readonly sha: string;
}

/** A relative path from the repository that is safe to write below a folder. */
const safeRel = (value: unknown): value is string =>
  typeof value === "string" &&
  value !== "" &&
  !value.startsWith("/") &&
  !value.includes("\0") &&
  !value
    .split("/")
    .some((part) => part === "" || part === ".." || part === "." || part.toLowerCase() === ".git");

const blobSha = (file: string, size: number): string | null => {
  try {
    const content = readFileSync(file);
    return createHash("sha1").update(`blob ${size}\0`).update(content).digest("hex");
  } catch {
    return null;
  }
};

async function resolveCommit(repo: string, ref: string) {
  const result = await run("gh", [
    "api",
    `repos/${repo}/branches/${encodeURIComponent(ref)}`,
    "--jq",
    '.commit.sha + " " + .commit.commit.tree.sha',
  ]);
  if (isFailure(result)) return result;
  const [commit, tree] = result.stdout.trim().split(" ");
  return { commit: commit ?? "", tree: tree ?? "" };
}

async function remoteFiles(repo: string, ref: string, source: string) {
  const resolved = await resolveCommit(repo, ref);
  if (isFailure(resolved)) return resolved;
  const result = await run("gh", ["api", `repos/${repo}/git/trees/${resolved.tree}?recursive=1`], {
    maxBuffer: 64 * 1024 * 1024,
    timeout: 90_000,
  });
  if (isFailure(result)) return result;
  const parsed = JSON.parse(result.stdout) as {
    truncated?: boolean;
    tree: Array<{ path: string; type: string; mode: string; size?: number; sha: string }>;
  };
  const prefix = source ? `${source}/` : "";
  const files: RemoteFile[] = [];
  for (const entry of parsed.tree) {
    if (entry.type !== "blob" || (entry.mode !== "100644" && entry.mode !== "100755")) continue;
    if (!entry.path.startsWith(prefix)) continue;
    const rel = entry.path.slice(prefix.length);
    if (!safeRel(rel)) continue;
    files.push({ path: entry.path, rel, size: entry.size ?? 0, sha: entry.sha });
    if (files.length >= 20_000) break;
  }
  return { files, truncated: parsed.truncated === true, commit: resolved.commit };
}

/** Every folder of `target` that already exists must stay inside `root`, so a symlink cannot redirect a write. */
function staysInside(root: string, rel: string): boolean {
  const target = resolve(root, rel);
  let existing = target;
  while (!existsSync(existing)) existing = dirname(existing);
  try {
    const real = realpathSync(existing);
    return real === root || real.startsWith(root + sep);
  } catch {
    return false;
  }
}

function downloadTarball(repo: string, commit: string, file: string): Promise<Failure | null> {
  return new Promise((resolvePromise) => {
    const fd = openSync(file, "w");
    const child = spawn("gh", ["api", `repos/${repo}/tarball/${commit}`], {
      stdio: ["ignore", fd, "pipe"],
      env: { ...process.env, GH_PROMPT_DISABLED: "1" },
    });
    let stderr = "";
    child.stderr?.on("data", (chunk: Buffer) => (stderr += chunk.toString()));
    // @effect-diagnostics-next-line globalTimers:off
    const timer = setTimeout(() => child.kill(), 180_000);
    child.on("close", (code) => {
      clearTimeout(timer);
      closeSync(fd);
      resolvePromise(code === 0 ? null : fail(stderr.trim() || "Download failed."));
    });
  });
}

function firstTarMember(file: string): Promise<string | null> {
  return new Promise((resolvePromise) => {
    const child = spawn("tar", ["-tzf", file], { stdio: ["ignore", "pipe", "ignore"] });
    let buffer = "";
    child.stdout.on("data", (chunk: Buffer) => {
      buffer += chunk.toString();
      const end = buffer.indexOf("\n");
      if (end >= 0) {
        resolvePromise(buffer.slice(0, end).replace(/\/$/, ""));
        child.kill();
      }
    });
    child.on("close", () => resolvePromise(null));
  });
}

/** Looks up to two levels below `root` for the checkout of `repo`, so a parent folder of the repo can be resolved. */
async function findRepoFolder(root: string, repo: string): Promise<string | null> {
  const queue: Array<{ dir: string; depth: number }> = [{ dir: root, depth: 0 }];
  let visited = 0;
  while (queue.length > 0 && visited < 300) {
    const { dir, depth } = queue.shift()!;
    visited += 1;
    if (existsSync(join(dir, ".git"))) {
      const origin = await run("git", ["remote", "get-url", "origin"], { cwd: dir });
      if (!isFailure(origin) && repoFromRemote(origin.stdout) === repo) return dir;
      continue;
    }
    if (depth >= 2) continue;
    let children: string[] = [];
    try {
      children = readdirSync(dir, { withFileTypes: true })
        .filter(
          (entry) =>
            entry.isDirectory() && !entry.isSymbolicLink() && entry.name !== "node_modules",
        )
        .map((entry) => entry.name);
    } catch {
      continue;
    }
    for (const name of children) queue.push({ dir: join(dir, name), depth: depth + 1 });
  }
  return null;
}

/** Publishes the current branch; a branch without upstream is pushed with `-u origin`. */
async function pushCurrent(cwd: string) {
  const upstream = await run("git", ["rev-parse", "--abbrev-ref", "@{u}"], { cwd });
  if (!isFailure(upstream)) return run("git", ["push"], { cwd, timeout: 120_000 });
  const branch = await run("git", ["branch", "--show-current"], { cwd });
  if (isFailure(branch) || !branch.stdout.trim()) return fail("No branch to publish.");
  return run("git", ["push", "-u", "origin", branch.stdout.trim()], { cwd, timeout: 120_000 });
}

async function handle(request: Record<string, unknown>): Promise<Success | Failure> {
  const ok = (data: unknown): Success => ({ ok: true, data });
  switch (request.op) {
    case "status": {
      const result = await run("gh", ["api", "user", "--jq", ".login"]);
      return isFailure(result) ? result : ok({ login: result.stdout.trim() });
    }
    case "repos": {
      const owner = request.owner;
      if (owner !== undefined && !(typeof owner === "string" && OWNER.test(owner))) {
        return fail("Invalid owner.");
      }
      const result = await run("gh", [
        "repo",
        "list",
        ...(owner ? [owner] : []),
        "--limit",
        "100",
        "--json",
        "nameWithOwner,description,isPrivate,isFork,pushedAt,url,defaultBranchRef",
      ]);
      return isFailure(result) ? result : ok(JSON.parse(result.stdout));
    }
    case "repo": {
      if (!safeRepo(request.repo)) return fail("Use owner/name.");
      const result = await run("gh", [
        "repo",
        "view",
        request.repo,
        "--json",
        "nameWithOwner,description,isPrivate,url,defaultBranchRef",
      ]);
      return isFailure(result) ? result : ok(JSON.parse(result.stdout));
    }
    case "branches": {
      if (!safeRepo(request.repo)) return fail("Use owner/name.");
      const result = await run(
        "gh",
        ["api", `repos/${request.repo}/branches?per_page=100`, "--paginate", "--jq", ".[].name"],
        { timeout: 60_000 },
      );
      return isFailure(result) ? result : ok(result.stdout.split("\n").filter(Boolean));
    }
    case "tree": {
      const { repo, ref, path } = request;
      if (!safeRepo(repo) || !safeRef(ref) || !safePath(path ?? ""))
        return fail("Invalid request.");
      const result = await run("gh", ["api", contentsUrl(repo, String(path ?? ""), ref)]);
      if (isFailure(result)) return result;
      const parsed = JSON.parse(result.stdout) as unknown;
      if (!Array.isArray(parsed)) return ok({ kind: "file" });
      const entries = (parsed as Array<{ name: string; path: string; type: string; size: number }>)
        .map((entry) => ({
          name: entry.name,
          path: entry.path,
          type: entry.type === "dir" ? "dir" : "file",
          size: entry.size,
        }))
        .toSorted((a, b) =>
          a.type === b.type ? a.name.localeCompare(b.name) : a.type === "dir" ? -1 : 1,
        );
      return ok({ kind: "dir", entries });
    }
    case "file": {
      const { repo, ref, path } = request;
      if (!safeRepo(repo) || !safeRef(ref) || !safePath(path) || path === "") {
        return fail("Invalid request.");
      }
      const result = await run(
        "gh",
        ["api", "-H", "Accept: application/vnd.github.raw", contentsUrl(repo, path, ref)],
        { maxBuffer: 1_500_000 },
      );
      if (isFailure(result))
        return fail(
          result.error.includes("maxBuffer") ? "File is too large to preview." : result.error,
        );
      if (result.stdout.includes("\0")) return ok({ binary: true });
      return ok({ binary: false, text: result.stdout });
    }
    case "local": {
      if (!safeDir(request.cwd))
        return ok({ exists: false, repo: null, branch: null, branches: [] });
      const cwd = request.cwd;
      const remote = await run("git", ["remote", "get-url", "origin"], { cwd });
      const branch = await run("git", ["branch", "--show-current"], { cwd });
      const branches = await run(
        "git",
        ["for-each-ref", "--format=%(refname:short)", "refs/heads"],
        { cwd },
      );
      return ok({
        exists: true,
        repo: isFailure(remote) ? null : repoFromRemote(remote.stdout),
        branch: isFailure(branch) ? null : branch.stdout.trim() || null,
        branches: isFailure(branches) ? [] : branches.stdout.split("\n").filter(Boolean),
      });
    }
    case "fetch": {
      if (!safeDir(request.cwd)) return fail("Not a folder.");
      const result = await run("git", ["fetch", "--prune", "origin"], {
        cwd: request.cwd,
        timeout: 120_000,
      });
      return isFailure(result) ? result : ok(null);
    }
    case "switch": {
      if (!safeDir(request.cwd) || !safeRef(request.branch)) return fail("Invalid request.");
      const result = await run("git", ["switch", request.branch], { cwd: request.cwd });
      return isFailure(result) ? result : ok(null);
    }
    case "createBranch": {
      const { cwd, name, base, push } = request;
      if (!safeDir(cwd) || !safeRef(name) || (base !== undefined && !safeRef(base))) {
        return fail("Invalid request.");
      }
      const locals = await run("git", ["for-each-ref", "--format=%(refname:short)", "refs/heads"], {
        cwd,
      });
      if (isFailure(locals)) return locals;
      const names = locals.stdout.split("\n");
      if (names.includes(name)) return fail("A local branch with this name exists.");
      const start = base === undefined ? [] : [names.includes(base) ? base : `origin/${base}`];
      const created = await run("git", ["switch", "--no-track", "-c", name, ...start], { cwd });
      if (isFailure(created)) return created;
      if (push === true) {
        const pushed = await run("git", ["push", "-u", "origin", name], { cwd, timeout: 120_000 });
        if (isFailure(pushed))
          return fail(`Branch created, but publishing failed: ${pushed.error}`);
      }
      return ok(null);
    }
    case "createRemoteBranch": {
      const { repo, name, base } = request;
      if (!safeRepo(repo) || !safeRef(name) || !safeRef(base)) return fail("Invalid request.");
      const sha = await run("gh", [
        "api",
        `repos/${repo}/branches/${encodeURIComponent(base)}`,
        "--jq",
        ".commit.sha",
      ]);
      if (isFailure(sha)) return sha;
      const created = await run("gh", [
        "api",
        `repos/${repo}/git/refs`,
        "-f",
        `ref=refs/heads/${name}`,
        "-f",
        `sha=${sha.stdout.trim()}`,
      ]);
      return isFailure(created) ? created : ok(null);
    }
    case "createRepo": {
      const { name, visibility, description, cwd, parent } = request;
      if (typeof name !== "string" || !OWNER.test(name) || name.startsWith("-")) {
        return fail("Use letters, digits, dots, dashes and underscores for the name.");
      }
      if (visibility !== "public" && visibility !== "private") return fail("Invalid visibility.");
      const text =
        typeof description === "string" && description.trim()
          ? [`--description=${description.trim().slice(0, 300)}`]
          : [];
      if (cwd !== undefined) {
        if (!safeDir(cwd)) return fail("Not a folder.");
        const origin = await run("git", ["remote", "get-url", "origin"], { cwd });
        if (!isFailure(origin)) return fail("This folder already has an origin remote.");
        const inside = await run("git", ["rev-parse", "--git-dir"], { cwd });
        if (isFailure(inside)) {
          const init = await run("git", ["init"], { cwd });
          if (isFailure(init)) return init;
        }
        const hasCommit = !isFailure(await run("git", ["rev-parse", "HEAD"], { cwd }));
        const created = await run(
          "gh",
          [
            "repo",
            "create",
            name,
            `--${visibility}`,
            `--source=${cwd}`,
            "--remote=origin",
            ...(hasCommit ? ["--push"] : []),
            ...text,
          ],
          { timeout: 120_000 },
        );
        if (isFailure(created)) return created;
        const view = await run(
          "gh",
          ["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"],
          {
            cwd,
          },
        );
        return ok({ repo: isFailure(view) ? null : view.stdout.trim(), path: cwd });
      }
      if (!safeDir(parent)) return fail("Not a folder.");
      if (existsSync(`${parent}/${name}`)) return fail("A folder with this name already exists.");
      const created = await run(
        "gh",
        ["repo", "create", name, `--${visibility}`, "--add-readme", "--clone", ...text],
        { cwd: parent, timeout: 120_000 },
      );
      if (isFailure(created)) return created;
      const path = `${parent}/${name}`;
      const view = await run(
        "gh",
        ["repo", "view", "--json", "nameWithOwner", "--jq", ".nameWithOwner"],
        {
          cwd: path,
        },
      );
      return ok({ repo: isFailure(view) ? null : view.stdout.trim(), path });
    }
    case "deleteRepo": {
      if (!safeRepo(request.repo)) return fail("Use owner/name.");
      const result = await run("gh", ["repo", "delete", request.repo, "--yes"]);
      if (isFailure(result)) {
        return fail(
          /delete_repo/.test(result.error)
            ? "GitHub needs the delete permission. Run `gh auth refresh -h github.com -s delete_repo` in a terminal, then try again."
            : result.error,
        );
      }
      return ok(null);
    }
    case "importPlan": {
      const { repo, ref, source, cwd } = request;
      if (!safeRepo(repo) || !safeRef(ref) || !safePath(source ?? "") || !safeDir(cwd)) {
        return fail("Invalid request.");
      }
      const listing = await remoteFiles(repo, ref, String(source ?? "").replace(/\/+$/, ""));
      if (isFailure(listing)) return listing;
      const files = listing.files.map((file) => {
        const local = join(cwd, file.rel);
        let status: "new" | "same" | "changed" = "new";
        try {
          const stat = statSync(local);
          if (stat.isFile()) {
            status =
              stat.size === file.size && blobSha(local, file.size) === file.sha
                ? "same"
                : "changed";
          } else status = "changed";
        } catch {
          status = "new";
        }
        return { rel: file.rel, size: file.size, status };
      });
      return ok({ files, truncated: listing.truncated });
    }
    case "importFiles": {
      const { repo, ref, source, cwd, paths } = request;
      if (!safeRepo(repo) || !safeRef(ref) || !safePath(source ?? "") || !safeDir(cwd)) {
        return fail("Invalid request.");
      }
      if (
        !Array.isArray(paths) ||
        paths.length === 0 ||
        paths.length > 20_000 ||
        !paths.every(safeRel)
      ) {
        return fail("Nothing valid to pull.");
      }
      const root = realpathSync(cwd);
      if (!paths.every((rel) => staysInside(root, rel))) return fail("A path leaves the folder.");
      const sourcePath = String(source ?? "").replace(/\/+$/, "");
      const resolved = await resolveCommit(repo, ref);
      if (isFailure(resolved)) return resolved;
      const work = mkdtempSync(join(tmpdir(), "cc3-pull-"));
      try {
        const archive = join(work, "repo.tar.gz");
        const downloaded = await downloadTarball(repo, resolved.commit, archive);
        if (downloaded) return downloaded;
        const top = await firstTarMember(archive);
        if (!top) return fail("The download was empty.");
        const members = paths.map((rel) => `${top}/${sourcePath ? `${sourcePath}/` : ""}${rel}`);
        const list = join(work, "members.txt");
        writeFileSync(list, members.join("\n") + "\n");
        const strip = 1 + (sourcePath ? sourcePath.split("/").length : 0);
        const extracted = await run(
          "tar",
          [
            "-xzf",
            archive,
            "-C",
            root,
            `--strip-components=${strip}`,
            "--no-same-owner",
            "-T",
            list,
          ],
          { timeout: 120_000 },
        );
        if (isFailure(extracted)) return extracted;
        return ok({ count: paths.length });
      } finally {
        rmSync(work, { recursive: true, force: true });
      }
    }
    case "findRepoFolder": {
      if (!safeDir(request.cwd) || !safeRepo(request.repo)) return fail("Invalid request.");
      return ok({ path: await findRepoFolder(request.cwd, request.repo) });
    }
    case "commits": {
      const { repo, ref, page } = request;
      if (!safeRepo(repo) || !safeRef(ref)) return fail("Invalid request.");
      const pageNumber =
        typeof page === "number" && page >= 1 && page <= 1000 ? Math.floor(page) : 1;
      const result = await run("gh", [
        "api",
        `repos/${repo}/commits?sha=${encodeURIComponent(ref)}&per_page=30&page=${pageNumber}`,
      ]);
      if (isFailure(result)) return result;
      const parsed = JSON.parse(result.stdout) as Array<{
        sha: string;
        commit: { message: string; author?: { name?: string; date?: string } };
        author?: { login?: string } | null;
      }>;
      return ok(
        parsed.map((entry) => ({
          sha: entry.sha,
          message: entry.commit.message,
          author: entry.author?.login ?? entry.commit.author?.name ?? "",
          date: entry.commit.author?.date ?? "",
        })),
      );
    }
    case "commit": {
      const { repo, sha } = request;
      if (!safeRepo(repo) || typeof sha !== "string" || !/^[0-9a-f]{7,40}$/.test(sha)) {
        return fail("Invalid request.");
      }
      const result = await run(
        "gh",
        ["api", "-H", "Accept: application/vnd.github.diff", `repos/${repo}/commits/${sha}`],
        { maxBuffer: 4 * 1024 * 1024 },
      );
      if (isFailure(result)) {
        return fail(
          result.error.includes("maxBuffer") ? "This commit is too large to show." : result.error,
        );
      }
      return ok({ diff: result.stdout });
    }
    case "wtStatus": {
      if (!safeDir(request.cwd)) return fail("Not a folder.");
      const result = await run(
        "git",
        ["status", "--porcelain=v1", "-z", "-b", "--untracked-files=all"],
        { cwd: request.cwd },
      );
      if (isFailure(result)) return result;
      const records = result.stdout.split("\0");
      const head = records.shift() ?? "";
      const branchLine = head.replace(/^## /, "");
      const track = /\[(?:ahead (\d+))?(?:, )?(?:behind (\d+))?\]/.exec(branchLine);
      const name = branchLine.replace(/ \[.*\]$/, "").split("...");
      const files: Array<{ path: string; x: string; y: string }> = [];
      for (let index = 0; index < records.length; index += 1) {
        const record = records[index] ?? "";
        if (record.length < 4) continue;
        const x = record[0] ?? " ";
        const y = record[1] ?? " ";
        files.push({ path: record.slice(3), x, y });
        if (x === "R" || x === "C") index += 1;
      }
      return ok({
        branch: /^HEAD \(no branch\)|^No commits yet on /.test(name[0] ?? "")
          ? (name[0] ?? "").replace(/^No commits yet on /, "")
          : (name[0] ?? ""),
        upstream: name[1] ?? null,
        ahead: Number(track?.[1] ?? 0),
        behind: Number(track?.[2] ?? 0),
        files,
      });
    }
    case "wtDiff": {
      const { cwd, path, untracked } = request;
      if (!safeDir(cwd) || !safePath(path) || path === "") return fail("Invalid request.");
      const args =
        untracked === true
          ? ["diff", "--no-color", "--no-index", "--", "/dev/null", path]
          : ["diff", "--no-color", "HEAD", "--", path];
      const result = await run("git", args, {
        cwd,
        okExit: [1],
        maxBuffer: 4 * 1024 * 1024,
      });
      if (isFailure(result)) {
        // A repository without commits has no HEAD to compare with.
        const fallback = await run("git", ["diff", "--no-color", "--cached", "--", path], { cwd });
        return isFailure(fallback) ? result : ok({ diff: fallback.stdout });
      }
      return ok({ diff: result.stdout });
    }
    case "wtCommit": {
      const { cwd, message, paths, push } = request;
      if (!safeDir(cwd) || typeof message !== "string" || !message.trim()) {
        return fail("Write a commit message.");
      }
      if (
        !Array.isArray(paths) ||
        paths.length === 0 ||
        !paths.every((p) => safePath(p) && p !== "")
      ) {
        return fail("Select the files to commit.");
      }
      await run("git", ["reset", "-q"], { cwd });
      const added = await run("git", ["add", "-A", "--", ...(paths as string[])], { cwd });
      if (isFailure(added)) return added;
      const committed = await run("git", ["commit", "-m", message.trim()], {
        cwd,
        timeout: 120_000,
      });
      if (isFailure(committed)) return committed;
      if (push === true) {
        const pushed = await pushCurrent(cwd);
        if (isFailure(pushed)) return fail(`Committed, but pushing failed: ${pushed.error}`);
      }
      return ok(null);
    }
    case "push": {
      if (!safeDir(request.cwd)) return fail("Not a folder.");
      const pushed = await pushCurrent(request.cwd);
      return isFailure(pushed) ? pushed : ok(null);
    }
    case "pull": {
      if (!safeDir(request.cwd)) return fail("Not a folder.");
      const result = await run("git", ["pull", "--ff-only"], {
        cwd: request.cwd,
        timeout: 120_000,
      });
      return isFailure(result)
        ? result
        : ok({ message: result.stdout.trim().split("\n")[0] ?? "" });
    }
    case "localBranches": {
      if (!safeDir(request.cwd)) return fail("Not a folder.");
      const result = await run(
        "git",
        [
          "for-each-ref",
          "--format=%(HEAD)\t%(refname:short)\t%(upstream:short)\t%(upstream:track)\t%(committerdate:relative)\t%(subject)",
          "refs/heads",
        ],
        { cwd: request.cwd },
      );
      if (isFailure(result)) return result;
      return ok(
        result.stdout
          .split("\n")
          .filter(Boolean)
          .map((line) => {
            const [head, name, upstream, track, when, subject] = line.split("\t");
            return {
              current: head === "*",
              name: name ?? "",
              upstream: upstream || null,
              track: track ?? "",
              when: when ?? "",
              subject: subject ?? "",
            };
          }),
      );
    }
    case "worktrees": {
      if (!safeDir(request.cwd)) return fail("Not a folder.");
      const result = await run("git", ["worktree", "list", "--porcelain"], { cwd: request.cwd });
      if (isFailure(result)) return result;
      const list: Array<{ path: string; branch: string | null; main: boolean }> = [];
      for (const block of result.stdout.split("\n\n")) {
        const lines = block.split("\n");
        const path = lines.find((line) => line.startsWith("worktree "))?.slice(9);
        if (!path) continue;
        const branch = lines.find((line) => line.startsWith("branch "))?.slice(7) ?? null;
        list.push({
          path,
          branch: branch ? branch.replace(/^refs\/heads\//, "") : null,
          main: list.length === 0,
        });
      }
      return ok(list);
    }
    case "worktreeAdd": {
      const { cwd, branch, base, create, path } = request;
      if (
        !safeDir(cwd) ||
        !safeRef(branch) ||
        (base !== undefined && !safeRef(base)) ||
        typeof path !== "string" ||
        !path.startsWith("/") ||
        path.split("/").includes("..") ||
        existsSync(path) ||
        !safeDir(dirname(path))
      ) {
        return fail("Choose a new folder inside an existing one.");
      }
      const args =
        create === true
          ? ["worktree", "add", "-b", branch, path, ...(base ? [base] : [])]
          : ["worktree", "add", path, branch];
      const result = await run("git", args, { cwd, timeout: 120_000 });
      return isFailure(result) ? result : ok({ path });
    }
    case "worktreeRemove": {
      const { cwd, path } = request;
      if (!safeDir(cwd) || typeof path !== "string" || !path.startsWith("/")) {
        return fail("Invalid request.");
      }
      const listed = await run("git", ["worktree", "list", "--porcelain"], { cwd });
      if (isFailure(listed)) return listed;
      const known = listed.stdout
        .split("\n")
        .filter((line) => line.startsWith("worktree "))
        .map((line) => line.slice(9));
      if (known[0] === path || !known.includes(path)) return fail("Not a linked worktree.");
      const result = await run("git", ["worktree", "remove", path], { cwd });
      return isFailure(result) ? result : ok(null);
    }
    case "deleteBranch": {
      const { cwd, name, remote } = request;
      if (!safeDir(cwd) || !safeRef(name)) return fail("Invalid request.");
      const local = await run("git", ["branch", "-d", name], { cwd });
      if (isFailure(local)) return local;
      if (remote === true) {
        const gone = await run("git", ["push", "origin", "--delete", name], {
          cwd,
          timeout: 120_000,
        });
        if (isFailure(gone)) return fail(`Deleted locally, but not on GitHub: ${gone.error}`);
      }
      return ok(null);
    }
    case "prs": {
      const { repo, state } = request;
      if (!safeRepo(repo)) return fail("Use owner/name.");
      const wanted = state === "closed" || state === "merged" || state === "all" ? state : "open";
      const result = await run("gh", [
        "pr",
        "list",
        "-R",
        repo,
        "--state",
        wanted,
        "--limit",
        "30",
        "--json",
        "number,title,state,isDraft,headRefName,baseRefName,author,url,reviewDecision,statusCheckRollup,updatedAt",
      ]);
      return isFailure(result) ? result : ok(JSON.parse(result.stdout));
    }
    case "prCreate": {
      const { cwd, title, body, base, draft } = request;
      if (!safeDir(cwd) || typeof title !== "string" || !title.trim() || !safeRef(base)) {
        return fail("Give the pull request a title.");
      }
      const pushed = await pushCurrent(cwd);
      if (isFailure(pushed)) return fail(`Publishing the branch failed: ${pushed.error}`);
      const result = await run(
        "gh",
        [
          "pr",
          "create",
          `--title=${title.trim()}`,
          `--body=${typeof body === "string" ? body : ""}`,
          `--base=${base}`,
          ...(draft === true ? ["--draft"] : []),
        ],
        { cwd, timeout: 120_000 },
      );
      return isFailure(result)
        ? result
        : ok({ url: result.stdout.trim().split("\n").at(-1) ?? "" });
    }
    case "prCheckout": {
      const { cwd, number } = request;
      if (!safeDir(cwd) || !Number.isInteger(number) || (number as number) < 1) {
        return fail("Invalid request.");
      }
      const result = await run("gh", ["pr", "checkout", String(number)], { cwd, timeout: 120_000 });
      return isFailure(result) ? result : ok(null);
    }
    case "prMerge": {
      const { repo, number, method, deleteBranch } = request;
      if (!safeRepo(repo) || !Number.isInteger(number) || (number as number) < 1) {
        return fail("Invalid request.");
      }
      const flag = method === "merge" ? "--merge" : method === "rebase" ? "--rebase" : "--squash";
      const result = await run(
        "gh",
        [
          "pr",
          "merge",
          String(number),
          "-R",
          repo,
          flag,
          ...(deleteBranch === true ? ["--delete-branch"] : []),
        ],
        { timeout: 120_000 },
      );
      return isFailure(result) ? result : ok(null);
    }
    default:
      return fail("Unknown operation.");
  }
}

export function installCc3GitHub(): void {
  Electron.ipcMain.removeHandler(CC3_GITHUB_CHANNEL);
  Electron.ipcMain.handle(CC3_GITHUB_CHANNEL, (_event, request: unknown) =>
    request && typeof request === "object"
      ? handle(request as Record<string, unknown>).catch((error: unknown) =>
          fail(error instanceof Error ? error.message : "Failed."),
        )
      : fail("Invalid request."),
  );
}
