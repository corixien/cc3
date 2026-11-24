// @effect-diagnostics nodeBuiltinImport:off -- A fixed set of short-lived gh and git calls inside one IPC handler; no Effect runtime is involved.
// CC3: backend of the GitHub panel. Runs a fixed set of read operations through the user's own `gh` and `git`,
// with every argument validated; nothing here takes a free-form command.
import * as Electron from "electron";
import { execFile } from "node:child_process";
import { existsSync, statSync } from "node:fs";

export const CC3_GITHUB_CHANNEL = "cc3:github";

const REPO = /^[\w.-]+\/[\w.-]+$/;
const OWNER = /^[\w.-]+$/;
const REF = /^[\w./@+-]+$/;

type Failure = { readonly ok: false; readonly error: string };
type Success = { readonly ok: true; readonly data: unknown };

const fail = (error: string): Failure => ({ ok: false, error });

function run(
  command: "gh" | "git",
  args: readonly string[],
  options: { cwd?: string; maxBuffer?: number; timeout?: number } = {},
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
      if (!safeDir(request.cwd)) return ok({ repo: null, branch: null, branches: [] });
      const cwd = request.cwd;
      const remote = await run("git", ["remote", "get-url", "origin"], { cwd });
      const branch = await run("git", ["branch", "--show-current"], { cwd });
      const branches = await run(
        "git",
        ["for-each-ref", "--format=%(refname:short)", "refs/heads"],
        { cwd },
      );
      return ok({
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
