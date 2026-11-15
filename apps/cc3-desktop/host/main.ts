// Node sidecar: serves apps/desktop's SSH IPC methods over stdio, one JSON object per line.
//   in : { id, channel, arg }   out: { id, result } | { id, error } | { event, payload }
import * as NodeHttpClient from "@effect/platform-node/NodeHttpClient";
import * as NodeCrypto from "@effect/platform-node/NodeCrypto";
import * as NodeRuntime from "@effect/platform-node/NodeRuntime";
import * as NodeServices from "@effect/platform-node/NodeServices";
import * as NetService from "@t3tools/shared/Net";
import * as Effect from "effect/Effect";
import * as Layer from "effect/Layer";
import * as Option from "effect/Option";
import * as NodePath from "node:path";
import * as NodeReadline from "node:readline";

import * as DesktopIpc from "../../desktop/src/ipc/DesktopIpc.ts";
import * as sshMethods from "../../desktop/src/ipc/methods/sshEnvironment.ts";
import * as ElectronWindow from "../../desktop/src/electron/ElectronWindow.ts";
import * as DesktopSshEnvironment from "../../desktop/src/ssh/DesktopSshEnvironment.ts";
import * as DesktopSshPasswordPrompts from "../../desktop/src/ssh/DesktopSshPasswordPrompts.ts";
import serverPackageJson from "../../server/package.json" with { type: "json" };

// Effect's default logger writes to stdout, which carries the protocol.
console.log = console.error;

const send = (message: unknown) => process.stdout.write(`${JSON.stringify(message)}\n`);

const handlers = new Map<string, DesktopIpc.DesktopIpcHandleListener>();
const ipcMain: DesktopIpc.DesktopIpcMain = {
  removeHandler: (channel) => void handlers.delete(channel),
  handle: (channel, listener) => void handlers.set(channel, listener),
  removeAllListeners: () => {},
  on: () => {},
};

// The renderer lives in Rust; this is the slice of BrowserWindow the SSH prompts touch.
const window = {
  isDestroyed: () => false,
  isMinimized: () => false,
  restore: () => {},
  focus: () => {},
  once: () => {},
  removeListener: () => {},
  webContents: { send: (event: string, payload: unknown) => send({ event, payload }) },
};

const layerWindow = Layer.succeed(ElectronWindow.ElectronWindow, {
  main: Effect.succeed(Option.some(window)),
} as never);

const home = process.env.CC3_HOME ?? NodePath.join(process.env.HOME ?? ".", ".cc3");

const layerRuntime = Layer.mergeAll(NodeServices.layer, NodeCrypto.layer).pipe(
  Layer.provideMerge(NodeHttpClient.layerUndici),
  Layer.provideMerge(NetService.layer),
);

const layerSsh = Layer.unwrap(
  Effect.gen(function* () {
    return DesktopSshEnvironment.layer({
      resolveCliRunner: Effect.succeed({ archiveVersion: serverPackageJson.version }),
      resolveLauncherId: yield* Effect.cached(
        DesktopSshEnvironment.readOrCreateLauncherId(
          NodePath.join(home, "userdata", "desktop", "ssh-launcher-id"),
        ),
      ),
    });
  }),
).pipe(Layer.provideMerge(DesktopSshPasswordPrompts.layer()));

const layerServices = Layer.mergeAll(
  Layer.succeed(
    DesktopIpc.DesktopIpc,
    DesktopIpc.make(ipcMain, () => Effect.succeed(true)),
  ),
  layerSsh,
).pipe(Layer.provideMerge(layerWindow), Layer.provideMerge(layerRuntime));

const program = Effect.gen(function* () {
  const ipc = yield* DesktopIpc.DesktopIpc;
  yield* Effect.forEach(Object.values(sshMethods), (method) => ipc.handle(method), {
    discard: true,
  }).pipe(Effect.catchDefect(Effect.die));

  const lines = NodeReadline.createInterface({ input: process.stdin });
  lines.on("line", (line) => {
    const { id, channel, arg } = JSON.parse(line) as { id: number; channel: string; arg: unknown };
    const handler = handlers.get(channel);
    if (!handler) return send({ id, error: `no handler for ${channel}` });
    Promise.resolve(handler({ sender: { id: 0 } }, arg)).then(
      (result) => send({ id, result: result ?? null }),
      (error) => send({ id, error: String(error?.message ?? error) }),
    );
  });
  yield* Effect.callback<never>((_resume) => {
    lines.on("close", () => process.exit(0));
  });
});

NodeRuntime.runMain(Effect.scoped(program).pipe(Effect.provide(layerServices)));
