// Injected into every page of the main window. Implements window.desktopBridge
// (packages/contracts DesktopBridge) on top of Tauri IPC. Not bundled: plain ES2020.
(() => {
  if (window.desktopBridge) return;
  const init = window.__CC3__ || {};
  const native = (op, ...args) => window.__TAURI_INTERNALS__.invoke("cc3_native", { op, args });
  const host = (channel, arg = null) =>
    window.__TAURI_INTERNALS__.invoke("cc3_host", { channel, arg });
  const listeners = new Map();
  const on = (channel, fn) => {
    if (!listeners.has(channel)) listeners.set(channel, new Set());
    listeners.get(channel).add(fn);
    return () => listeners.get(channel)?.delete(fn);
  };
  const dispatch = (channel, payload) => listeners.get(channel)?.forEach((fn) => fn(payload));
  const noop = () => () => {};
  const unsupported = (name) => () =>
    Promise.reject(new Error(`${name} is not supported in CC3 yet`));

  let fullscreen = false;
  let zoom = 1;
  window.__cc3emit = (channel, payload) => {
    if (channel === "fullscreen") {
      if (fullscreen !== payload) {
        fullscreen = payload;
        dispatch("fullscreen", payload);
      }
    } else if (channel === "zoom-step") {
      zoom =
        payload === "zoom-reset"
          ? 1
          : Math.min(5, Math.max(0.25, zoom + (payload === "zoom-in" ? 0.1 : -0.1)));
      native("set_zoom", zoom);
    } else dispatch(channel, payload);
  };

  const storeJson = async (name, fallback = null) => {
    const raw = await native("store_read", name);
    if (raw == null) return fallback;
    try {
      return JSON.parse(raw);
    } catch {
      return fallback;
    }
  };
  const storeSet = (name, value) => native("store_write", name, JSON.stringify(value));

  let closeMenu = null;
  const showContextMenu = (items, position) =>
    new Promise((resolve) => {
      closeMenu?.(null);
      const host = document.createElement("div");
      host.style.cssText = "position:fixed;inset:0;z-index:2147483647";
      const menu = document.createElement("div");
      menu.setAttribute("role", "menu");
      menu.style.cssText =
        "position:fixed;min-width:180px;padding:4px;border-radius:8px;font:13px system-ui,sans-serif;" +
        "background:var(--popover,#1e1e1e);color:var(--popover-foreground,#eee);border:1px solid var(--border,#444);box-shadow:0 8px 24px #0006";
      const done = (value) => {
        closeMenu = null;
        host.remove();
        window.removeEventListener("keydown", onKey, true);
        resolve(value);
      };
      closeMenu = done;
      const onKey = (e) => {
        if (e.key === "Escape") {
          e.preventDefault();
          done(null);
        }
      };
      const render = (list, into) => {
        for (const it of list) {
          if (it.separatorBefore) {
            const s = document.createElement("div");
            s.style.cssText = "height:1px;margin:4px 0;background:var(--border,#444)";
            into.append(s);
          }
          const row = document.createElement("div");
          row.setAttribute("role", "menuitem");
          row.style.cssText =
            "position:relative;display:flex;gap:8px;align-items:center;padding:5px 10px;border-radius:5px;" +
            (it.header
              ? "opacity:.6;font-size:11px;"
              : it.disabled
                ? "opacity:.45;"
                : "cursor:default;") +
            (it.destructive ? "color:var(--destructive,#f87171);" : "");
          row.textContent = (it.checked ? "✓ " : "") + it.label;
          if (it.detail) {
            const d = document.createElement("span");
            d.textContent = it.detail;
            d.style.cssText = "margin-left:auto;opacity:.6";
            row.append(d);
          }
          if (!it.header && !it.disabled) {
            row.onmouseenter = () => (row.style.background = "var(--accent,#333)");
            row.onmouseleave = () => (row.style.background = "");
            if (it.children?.length) {
              const sub = document.createElement("div");
              sub.style.cssText =
                menu.style.cssText + ";position:absolute;left:100%;top:-4px;display:none";
              render(it.children, sub);
              row.append(sub);
              row.addEventListener("mouseenter", () => (sub.style.display = "block"));
              row.addEventListener("mouseleave", () => (sub.style.display = "none"));
            } else
              row.onclick = (e) => {
                e.stopPropagation();
                done(it.id);
              };
          }
          into.append(row);
        }
      };
      render(items, menu);
      host.append(menu);
      host.addEventListener("mousedown", (e) => {
        if (e.target === host) done(null);
      });
      host.addEventListener("contextmenu", (e) => {
        e.preventDefault();
        if (e.target === host) done(null);
      });
      document.body.append(host);
      window.addEventListener("keydown", onKey, true);
      const x = position?.x ?? window.__cc3Mouse?.x ?? 20,
        y = position?.y ?? window.__cc3Mouse?.y ?? 20;
      const r = menu.getBoundingClientRect();
      menu.style.left = Math.max(0, Math.min(x, innerWidth - r.width - 4)) + "px";
      menu.style.top = Math.max(0, Math.min(y, innerHeight - r.height - 4)) + "px";
    });
  addEventListener("mousemove", (e) => (window.__cc3Mouse = { x: e.clientX, y: e.clientY }), true);

  let bearer = null;
  const getBearer = async () => {
    if (bearer) return bearer;
    const boot = (init.bootstraps || [])[0];
    const body = new URLSearchParams({
      grant_type: "urn:ietf:params:oauth:grant-type:token-exchange",
      subject_token: boot.bootstrapToken,
      subject_token_type: "urn:t3:params:oauth:token-type:environment-bootstrap",
      requested_token_type: "urn:ietf:params:oauth:token-type:access_token",
      client_label: "CC3",
      client_device_type: "desktop",
    });
    const res = await fetch(new URL("/oauth/token", boot.httpBaseUrl), { method: "POST", body });
    if (!res.ok) throw new Error(`bearer bootstrap failed: ${res.status}`);
    bearer = (await res.json()).access_token;
    return bearer;
  };

  const arch = init.arch === "aarch64" ? "arm64" : init.arch === "x86_64" ? "x64" : "other";
  const updateState = {
    enabled: false,
    status: "disabled",
    channel: "latest",
    currentVersion: init.version || "0.0.0",
    hostArch: arch,
    appArch: arch,
    runningUnderArm64Translation: false,
    availableVersion: null,
    downloadedVersion: null,
    releaseNotes: [],
    omittedReleaseCount: 0,
    downloadPercent: null,
    checkedAt: null,
    message: null,
    errorContext: null,
    canRetry: false,
  };
  const exposureState = {
    mode: "local-only",
    endpointUrl: null,
    advertisedHost: null,
    tailscaleServeEnabled: false,
    tailscaleServePort: 443,
  };
  const wslState = {
    enabled: false,
    distro: null,
    available: false,
    wslOnly: false,
    distros: [],
    preflightError: null,
  };
  const cliState = { supported: false, installedPath: null, onPath: false, launcherPath: null };
  const snapState = {
    mode: "unavailable",
    shortcut: { kind: "both-shift-keys" },
    shortcutRegistered: false,
    shortcutMessage: null,
  };

  window.desktopBridge = {
    getAppBranding: () => ({ baseName: "CC3", stageLabel: "Alpha", displayName: "CC3 (Alpha)" }),
    getRelayTelemetryEnabled: () => false,
    getPathForFile: () => "",
    getClientPlatform: () => "linux",
    restartAudioService: async () => false,
    setNotificationBadge: async () => {},
    onNotificationBadgeClear: noop,
    onTrackpadScrollBegin: noop,
    onTrackpadScrollEnd: noop,
    revealWindow: () => native("reveal_window"),
    getSystemLocale: () => init.locale ?? null,
    getPreferredSystemLanguages: () => (init.locale ? [init.locale] : []),
    getPrimaryPasteEnabled: () => true,
    getLocalEnvironmentBootstraps: () => (localEnabled ? init.bootstraps || [] : []),
    getLocalEnvironmentEnabled: () => localEnabled,
    setLocalEnvironmentEnabled: async (enabled) => {
      localEnabled = enabled;
      await storeSet("local-environment-enabled.json", enabled);
    },
    getHostEnvironmentId: async () => null,
    getLocalEnvironmentBearerToken: getBearer,
    getClientSettings: () => storeJson("client-settings.json"),
    setClientSettings: (settings) => storeSet("client-settings.json", settings).then(() => {}),
    getConnectionCatalog: async () =>
      (await native("store_read", "connection-catalog.json")) ?? null,
    setConnectionCatalog: (catalog) => native("store_write", "connection-catalog.json", catalog),
    discoverSshHosts: () => host("desktop:discover-ssh-hosts"),
    resolveSshHost: (alias) => host("desktop:resolve-ssh-host", alias),
    requestSnapShotPermissions: async () => {},
    getSnapShotState: async () => snapState,
    setupSnapShot: async () => {},
    previewSnapShotConfig: async () => null,
    applySnapShotConfig: unsupported("applySnapShotConfig"),
    checkSnapShotShortcut: async () => ({
      available: false,
      message: "Snapshots are not supported in CC3 yet",
    }),
    setSnapShotShortcutSuppressed: async () => {},
    listPendingSnapShots: async () => [],
    readSnapShot: unsupported("readSnapShot"),
    setSnapShotAnimationDestination: async () => {},
    dismissSnapShotAnimation: async () => {},
    acknowledgeSnapShot: async () => {},
    ensureSshEnvironment: async (target, options) => {
      const result = await host("desktop:ensure-ssh-environment", {
        target,
        ...(options === undefined ? {} : { options }),
      });
      if (result?.type === "ssh-password-prompt-cancelled")
        throw new Error(result.message || "SSH authentication cancelled.");
      return result;
    },
    disconnectSshEnvironment: (target) => host("desktop:disconnect-ssh-environment", target),
    fetchSshEnvironmentDescriptor: (httpBaseUrl) =>
      host("desktop:fetch-ssh-environment-descriptor", { httpBaseUrl }),
    bootstrapSshBearerSession: (httpBaseUrl, credential) =>
      host("desktop:bootstrap-ssh-bearer-session", { httpBaseUrl, credential }),
    fetchSshSessionState: (httpBaseUrl, bearerToken) =>
      host("desktop:fetch-ssh-session-state", { httpBaseUrl, bearerToken }),
    onSshPasswordPrompt: (fn) => on("desktop:ssh-password-prompt", fn),
    onSshPasswordPromptSettled: (fn) => on("desktop:ssh-password-prompt-settled", fn),
    resolveSshPasswordPrompt: (requestId, password) =>
      host("desktop:resolve-ssh-password-prompt", { requestId, password }),
    getServerExposureState: async () => exposureState,
    setServerExposureMode: unsupported("setServerExposureMode"),
    setTailscaleServeEnabled: unsupported("setTailscaleServeEnabled"),
    getAdvertisedEndpoints: async () => [],
    getWslState: async () => wslState,
    setWslBackendEnabled: async () => wslState,
    setWslDistro: async () => wslState,
    setWslOnly: async () => wslState,
    pickFolder: (options) => native("pick_folder", options?.initialPath ?? null),
    toLocalBackendPath: async () => null,
    pickProjectFavicon: (initialPath) => native("pick_favicon", initialPath ?? null),
    pickThemeFiles: () => native("pick_theme_files"),
    setTheme: (theme) => native("set_theme", theme),
    showContextMenu,
    receiveProviderAuthCallback: unsupported("receiveProviderAuthCallback"),
    cancelProviderAuthCallback: async () => {},
    openExternal: (url) => native("open_external", url),
    readMedia: async (url) => {
      const r = await native("read_media", url);
      return r ? { ...r, data: new Uint8Array(r.data) } : null;
    },
    openSystemSettings: async () => false,
    checkSystemPermission: async () => true,
    probeRemoteEditors: async () => [],
    pasteAsText: async () => {
      const text = await navigator.clipboard.readText().catch(() => "");
      if (text) document.execCommand("insertText", false, text);
    },
    onMenuAction: (fn) => on("menu-action", fn),
    onSnapShotEvent: noop,
    onQuitShortcut: noop,
    getWindowFullscreenState: () => fullscreen,
    onWindowFullscreenStateChange: (fn) => on("fullscreen", fn),
    getWindowZoomFactor: () => zoom,
    getUpdateState: async () => updateState,
    setUpdateChannel: async () => updateState,
    checkForUpdate: async () => ({ checked: false, state: updateState }),
    downloadUpdate: async () => ({ accepted: false, completed: false, state: updateState }),
    installUpdate: async () => ({ accepted: false, completed: false, state: updateState }),
    onUpdateState: noop,
    cliCommand: {
      getState: async () => cliState,
      install: async () => cliState,
      uninstall: async () => cliState,
      addToPath: async () => cliState,
    },
    webLinks: { setReady: async () => {}, onOpen: (fn) => on("web-link-open", fn) },
    appActivation: {
      setReady: async () => {},
      complete: async () => {},
      onRequest: (fn) => on("app-activation", fn),
    },
    // `preview` is intentionally absent: the embedded browser needs Electron <webview>.
  };

  if (init.debugPort) {
    const log = (window.__cc3Log = (m) => {
      try {
        new Image().src =
          `http://127.0.0.1:${init.debugPort}/` + encodeURIComponent(String(m).slice(0, 400));
      } catch {}
    });
    log("bridge installed; tauri=" + typeof window.__TAURI_INTERNALS__);
    addEventListener("unhandledrejection", (e) =>
      log("unhandledrejection: " + (e.reason?.stack || e.reason)),
    );
    addEventListener("error", (e) => log("error: " + e.message));
    for (const [k, fn] of Object.entries(window.desktopBridge)) {
      if (typeof fn !== "function") continue;
      window.desktopBridge[k] = (...a) => {
        log("call " + k);
        let r;
        try {
          r = fn(...a);
        } catch (e) {
          log(`${k} threw ${e}`);
          throw e;
        }
        if (r?.then)
          r.then(
            () => log(`${k} ok`),
            (e) => log(`${k} rejected ${e}`),
          );
        return r;
      };
    }
  }

  if (init.debugPort && init.selfTest) {
    setTimeout(
      () =>
        window.desktopBridge.discoverSshHosts().then(
          (r) => window.__cc3Log?.("selftest " + JSON.stringify(r)),
          (e) => window.__cc3Log?.("selftest failed " + e),
        ),
      3000,
    );
  }

  let localEnabled = true;
  storeJson("local-environment-enabled.json", true).then((v) => {
    localEnabled = v !== false;
  });
})();
