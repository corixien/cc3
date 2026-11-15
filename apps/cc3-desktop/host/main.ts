// Node sidecar: serves apps/desktop's Electron-free IPC methods over stdio, one JSON object per line.
//   in : { id, channel, arg }   out: { id, result } | { id, error } | { event, payload }
import * as NodeHttpClient from "@effect/platform-node/NodeHttpClient";
import * as NodeCrypto from "@effect/platform-node/NodeCrypto";
import * as NodeRuntime from "@effect/platform-node/NodeRuntime";
import * as NodeServices from "@effect/platform-node/NodeServices";
import * as NetService from "@t3tools/shared/Net";
import * as Effect from "effect/Effect";
import * as Layer from "effect/Layer";
import * as Option from "effect/Option";
import * as NodeOS from "node:os";
import * as NodePath from "node:path";
import * as NodeReadline from "node:readline";
import * as Schema from "effect/Schema";
import {
  AdvertisedEndpoint,
  ClientSettingsSchema,
  DesktopServerExposureModeSchema,
  DesktopServerExposureStateSchema,
} from "@t3tools/contracts";

import * as DesktopIpc from "../../desktop/src/ipc/DesktopIpc.ts";
import * as sshMethods from "../../desktop/src/ipc/methods/sshEnvironment.ts";
import * as providerAuthMethods from "../../desktop/src/ipc/methods/providerAuth.ts";
import * as ElectronShell from "../../desktop/src/electron/ElectronShell.ts";
import * as cliMethods from "../../desktop/src/ipc/methods/cliCommand.ts";
import * as catalogMethods from "../../desktop/src/ipc/methods/connectionCatalog.ts";
import * as IpcChannels from "../../desktop/src/ipc/channels.ts";
import * as DesktopEnvironment from "../../desktop/src/app/DesktopEnvironment.ts";
import * as DesktopCliCommand from "../../desktop/src/app/DesktopCliCommand.ts";
import * as DesktopConnectionCatalogStore from "../../desktop/src/app/DesktopConnectionCatalogStore.ts";
import * as DesktopManagedPolicy from "../../desktop/src/app/DesktopManagedPolicy.ts";
import * as DesktopNetworkInterfaces from "../../desktop/src/backend/DesktopNetworkInterfaces.ts";
import * as DesktopServerExposure from "../../desktop/src/backend/DesktopServerExposure.ts";
import * as DesktopAppSettings from "../../desktop/src/settings/DesktopAppSettings.ts";
import * as DesktopClientSettings from "../../desktop/src/settings/DesktopClientSettings.ts";
import * as CC3SafeStorage from "./safeStorage.ts";
import * as DesktopSavedEnvironments from "../../desktop/src/settings/DesktopSavedEnvironments.ts";

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
  currentMainOrFirst: Effect.succeed(Option.none()),
} as never);

const layerShell = Layer.succeed(ElectronShell.ElectronShell, {
  openExternal: (url: string) =>
    Effect.sync(() => send({ event: "cc3:open-external", payload: url })),
} as never);

const home = process.env.CC3_HOME ?? NodePath.join(process.env.HOME ?? ".", ".cc3");

const layerRuntime = Layer.mergeAll(NodeServices.layer, NodeCrypto.layer).pipe(
  Layer.provideMerge(NodeHttpClient.layerUndici),
  Layer.provideMerge(NetService.layer),
);

const repoRoot = NodePath.resolve(import.meta.dirname, "../../..");

const layerEnvironment = DesktopEnvironment.layer({
  dirname: import.meta.dirname,
  homeDirectory: NodeOS.homedir(),
  platform: "linux",
  // oxlint-disable-next-line t3code/no-global-process-runtime -- standalone host, no Effect runtime services
  processArch: process.arch,
  appVersion: serverPackageJson.version,
  appPath: repoRoot,
  isPackaged: false,
  resourcesPath: repoRoot,
  runningUnderArm64Translation: false,
});

const layerBase = Layer.mergeAll(
  DesktopManagedPolicy.layer,
  DesktopAppSettings.layer,
  DesktopNetworkInterfaces.layer,
).pipe(Layer.provideMerge(layerEnvironment), Layer.provideMerge(CC3SafeStorage.layer));

const layerFoundation = Layer.mergeAll(
  DesktopClientSettings.layer,
  DesktopConnectionCatalogStore.layer.pipe(Layer.provideMerge(DesktopSavedEnvironments.layer)),
  DesktopCliCommand.layer,
  DesktopServerExposure.layer,
).pipe(Layer.provideMerge(layerBase));

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
  layerFoundation,
).pipe(
  Layer.provideMerge(layerWindow),
  Layer.provideMerge(layerShell),
  Layer.provideMerge(layerRuntime),
);

const program = Effect.gen(function* () {
  const ipc = yield* DesktopIpc.DesktopIpc;
  const exposure = yield* DesktopServerExposure.DesktopServerExposure;
  const clientSettings = yield* DesktopClientSettings.DesktopClientSettings;
  const appSettings = yield* DesktopAppSettings.DesktopAppSettings;

  const wrap = <A, S>(
    channel: string,
    payload: Schema.Codec<A, unknown>,
    result: S,
    handler: (input: A) => Effect.Effect<unknown>,
  ) => ({
    channel,
    handler: (raw: unknown) =>
      Schema.decodeUnknownEffect(payload)(raw).pipe(
        Effect.flatMap(handler),
        Effect.flatMap((value) => Schema.encodeUnknownEffect(result as never)(value)),
      ),
  });
  const relaunchBackend = (reason: string) =>
    Effect.sync(() => send({ event: "cc3:relaunch-backend", payload: reason }));

  const custom = [
    wrap(
      IpcChannels.GET_SERVER_EXPOSURE_STATE_CHANNEL,
      Schema.Void,
      DesktopServerExposureStateSchema,
      () => exposure.getState,
    ),
    wrap(
      IpcChannels.GET_ADVERTISED_ENDPOINTS_CHANNEL,
      Schema.Void,
      Schema.Array(AdvertisedEndpoint),
      () => exposure.getAdvertisedEndpoints,
    ),
    wrap(
      IpcChannels.GET_CLIENT_SETTINGS_CHANNEL,
      Schema.Void,
      Schema.NullOr(ClientSettingsSchema),
      () => clientSettings.get.pipe(Effect.map(Option.getOrNull)),
    ),
    wrap(IpcChannels.SET_CLIENT_SETTINGS_CHANNEL, ClientSettingsSchema, Schema.Void, (settings) =>
      clientSettings.set(settings),
    ),
    wrap(
      IpcChannels.SET_SERVER_EXPOSURE_MODE_CHANNEL,
      DesktopServerExposureModeSchema,
      DesktopServerExposureStateSchema,
      (mode) =>
        exposure.setMode(mode).pipe(
          Effect.tap((change) =>
            change.requiresRelaunch ? relaunchBackend(`mode=${mode}`) : Effect.void,
          ),
          Effect.map((change) => change.state),
        ),
    ),
    wrap(
      IpcChannels.SET_TAILSCALE_SERVE_ENABLED_CHANNEL,
      Schema.Struct({ enabled: Schema.Boolean, port: Schema.optionalKey(Schema.Number) }),
      DesktopServerExposureStateSchema,
      (input) =>
        exposure.setTailscaleServeEnabled(input).pipe(
          Effect.tap((change) =>
            change.requiresRelaunch ? relaunchBackend("tailscale-serve") : Effect.void,
          ),
          Effect.map((change) => change.state),
        ),
    ),
    wrap("cc3:configure-backend", Schema.Struct({ port: Schema.Number }), Schema.Any, ({ port }) =>
      appSettings.load.pipe(
        Effect.andThen(exposure.configureFromSettings({ port })),
        Effect.andThen(exposure.backendConfig),
        Effect.andThen((config) =>
          appSettings.get.pipe(Effect.map((settings) => ({ config, settings }))),
        ),
        Effect.map(({ config, settings }) => ({
          localEnvironmentEnabled: settings.localEnvironmentEnabled !== false,
          port: config.port,
          bindHost: config.bindHost,
          tailscaleServeEnabled: config.tailscaleServeEnabled,
          tailscaleServePort: config.tailscaleServePort,
        })),
      ),
    ),
    wrap("cc3:set-local-environment-enabled", Schema.Boolean, Schema.Void, (enabled) =>
      appSettings.setLocalEnvironmentEnabled(enabled).pipe(Effect.asVoid),
    ),
  ];

  yield* Effect.forEach(
    [
      ...Object.values(sshMethods),
      ...Object.values(providerAuthMethods),
      ...Object.values(cliMethods),
      ...Object.values(catalogMethods),
      ...custom,
    ],
    (method) => ipc.handle(method),
    { discard: true },
  ).pipe(Effect.catchDefect(Effect.die));

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
