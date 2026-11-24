// @effect-diagnostics nodeBuiltinImport:off -- A fixed set of short-lived gh and git calls inside one IPC handler; no Effect runtime is involved.
// CC3: backend of the GitHub panel. Runs a fixed set of read operations through the user's own `gh` and `git`,
// with every argument validated; nothing here takes a free-form command.
import * as Electron from "electron";
import { execFile } from "node:child_process";
import { statSync } from "node:fs";

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
