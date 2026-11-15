// Stages everything the Tauri bundle ships under src-tauri/resources:
//   node, server/ (pnpm deploy of apps/server), host/ (bundled desktop host + its two externals).
// Prerequisites: `pnpm vp run --filter @t3tools/web --filter t3 build` and `pnpm exec vp pack` here.
import * as NodeChildProcess from "node:child_process";
import * as NodeFS from "node:fs";
import * as NodeModule from "node:module";
import * as NodePath from "node:path";

const root = NodePath.resolve(import.meta.dirname, "../../..");
const appDir = NodePath.resolve(import.meta.dirname, "..");
const out = NodePath.join(appDir, "src-tauri/resources");

NodeFS.rmSync(out, { recursive: true, force: true });
NodeFS.mkdirSync(out, { recursive: true });

NodeChildProcess.execFileSync(
  "pnpm",
  [
    "--filter",
    "t3",
    "deploy",
    "--prod",
    "--config.node-linker=hoisted",
    NodePath.join(out, "server"),
  ],
  { cwd: root, stdio: "inherit" },
);

NodeFS.copyFileSync(process.execPath, NodePath.join(out, "node"));

const hostOut = NodePath.join(out, "host");
NodeFS.cpSync(NodePath.join(appDir, "host/dist"), hostOut, { recursive: true });
const requireFrom = (dir) => NodeModule.createRequire(NodePath.join(dir, "noop.js"));
const packageDir = (name, from) =>
  NodeFS.realpathSync(NodePath.dirname(requireFrom(from).resolve(`${name}/package.json`)));
const keyringDir = packageDir("@napi-rs/keyring", appDir);
const platformNodeDir = packageDir("@effect/platform-node", appDir);
const externals = {
  "@napi-rs/keyring": keyringDir,
  "@napi-rs/keyring-linux-x64-gnu": packageDir("@napi-rs/keyring-linux-x64-gnu", keyringDir),
  undici: packageDir("undici", platformNodeDir),
};
for (const [name, dir] of Object.entries(externals)) {
  NodeFS.cpSync(dir, NodePath.join(hostOut, "node_modules", name), {
    recursive: true,
    dereference: true,
  });
}
console.log(`staged ${out}`);
