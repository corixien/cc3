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

  // Frameless window: draw the caption buttons, polyfill the Window Controls Overlay the web UI
  // reads, and turn `.drag-region` into a native drag (WebKitGTK ignores -webkit-app-region).
  const CONTROLS_WIDTH = 138;
  const TITLEBAR_HEIGHT = 40;
  {
    const wco = new EventTarget();
    const rect = () => new DOMRect(0, 0, Math.max(0, innerWidth - CONTROLS_WIDTH), TITLEBAR_HEIGHT);
    Object.defineProperty(navigator, "windowControlsOverlay", {
      configurable: true,
      value: {
        visible: true,
        getTitlebarAreaRect: rect,
        addEventListener: (t, fn) => wco.addEventListener(t, fn),
        removeEventListener: (t, fn) => wco.removeEventListener(t, fn),
      },
    });
    addEventListener("resize", () => wco.dispatchEvent(new Event("geometrychange")));
  }
  const installChrome = () => {
    const style = document.createElement("style");
    style.textContent = `
      .wco {
        --native-titlebar-height: ${TITLEBAR_HEIGHT}px;
        --workspace-topbar-height: ${TITLEBAR_HEIGHT}px;
        --workspace-controls-top: 0px;
        --workspace-controls-left: 0.75rem;
        --workspace-controls-right: calc(${CONTROLS_WIDTH}px + 0.75rem);
        --workspace-native-controls-inset: calc(${CONTROLS_WIDTH}px + 0.75rem);
      }
      html { clip-path: inset(0 round 14px); }
      html.cc3-square { clip-path: none; }
      html::after { content: ""; position: fixed; inset: 0; border: 1px solid #252525;
        border-radius: 14px; pointer-events: none; z-index: 2147483646; }
      html.cc3-square::after { display: none; }
      #cc3-controls { position: fixed; top: 0; right: 0; z-index: 2147483647; display: flex;
        height: ${TITLEBAR_HEIGHT}px; color: #f8fafc; user-select: none; }
      #cc3-controls.light { color: #1f2937; }
      #cc3-controls button { all: unset; box-sizing: border-box; width: 46px; height: 100%;
        display: flex; align-items: center; justify-content: center; cursor: default; }
      #cc3-controls button:hover { background: rgba(127,127,127,.22); }
      #cc3-controls button.close:hover { background: #e81123; color: #fff; }
      #cc3-controls svg { width: 10px; height: 10px; stroke: currentColor; fill: none; stroke-width: 1; }
    `;
    document.documentElement.append(style);

    const bar = document.createElement("div");
    bar.id = "cc3-controls";
    const icons = {
      min: '<path d="M0 5.5h10"/>',
      max: '<rect x="0.5" y="0.5" width="9" height="9"/>',
      restore: '<path d="M2.5 2.5v-2h7v7h-2M0.5 2.5h7v7h-7z"/>',
      close: '<path d="M0 0l10 10M10 0L0 10"/>',
    };
    const button = (cls, icon, label, op) => {
      const b = document.createElement("button");
      b.className = cls;
      b.title = label;
      b.innerHTML = `<svg viewBox="0 0 10 10">${icon}</svg>`;
      b.addEventListener("click", () => native(op).then(refresh));
      return b;
    };
    const maxButton = button("max", icons.max, "Maximize", "win_toggle_maximize");
    bar.append(
      button("min", icons.min, "Minimize", "win_minimize"),
      maxButton,
      button("close", icons.close, "Close", "win_close"),
    );
    document.body.append(bar);

    function refresh() {
      native("win_is_maximized").then((maximized) => {
        maxButton.innerHTML = `<svg viewBox="0 0 10 10">${maximized ? icons.restore : icons.max}</svg>`;
        document.documentElement.classList.toggle("cc3-square", maximized || fullscreen);
      });
      bar.style.display = fullscreen ? "none" : "flex";
    }
    addEventListener("resize", refresh);
    refresh();

    const root = document.documentElement;
    const syncTheme = () => bar.classList.toggle("light", !root.classList.contains("dark"));
    new MutationObserver(syncTheme).observe(root, { attributes: true, attributeFilter: ["class"] });
    syncTheme();

    const INTERACTIVE =
      'button,input,textarea,select,a,summary,[role="button"],[role="menuitem"],[role="tab"],[contenteditable="true"],[class*="app-region:no-drag"],.no-drag,#cc3-controls';
    document.addEventListener(
      "mousedown",
      (e) => {
        if (e.button !== 0 || !(e.target instanceof Element)) return;
        if (!e.target.closest(".drag-region") || e.target.closest(INTERACTIVE)) return;
        e.preventDefault();
        if (e.detail === 2) native("win_toggle_maximize").then(refresh);
        else native("win_start_drag");
      },
      true,
    );
  };
  if (document.readyState === "loading") addEventListener("DOMContentLoaded", installChrome);
  else installChrome();

  // Desktop notifications go to the native alert card (alert.rs); a click comes back as an event.
  {
    const alerts = new Map();
    let nextAlert = 0;
    class CC3Notification extends EventTarget {
      static permission = "granted";
      static requestPermission = async () => "granted";
      constructor(title, options = {}) {
        super();
        this.title = title;
        this.body = options.body ?? "";
        this.tag = options.tag ?? "";
        this._id = ++nextAlert;
        alerts.set(this._id, this);
        native("show_alert", { id: this._id, title, body: this.body }).catch((e) =>
          window.__cc3Log?.(`show_alert failed ${e}`),
        );
      }
      close() {
        alerts.delete(this._id);
        native("close_alert").catch(() => {});
      }
    }
    Object.defineProperty(window, "Notification", {
      configurable: true,
      writable: true,
      value: CC3Notification,
    });
    on("notification-click", (id) => alerts.get(id)?.dispatchEvent(new Event("click")));
  }

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
  const wslState = {
    enabled: false,
    distro: null,
    available: false,
    wslOnly: false,
    distros: [],
    preflightError: null,
  };
  window.desktopBridge = {
    getAppBranding: () => ({ baseName: "CC3", stageLabel: "Alpha", displayName: "CC3 (Alpha)" }),
    getRelayTelemetryEnabled: () => false,
    getPathForFile: () => "",
    getClientPlatform: () => "linux",
    restartAudioService: async () => false,
    setNotificationBadge: (badge) => native("set_badge", badge),
    onNotificationBadgeClear: (fn) => on("notification-badge-clear", fn),
    onTrackpadScrollBegin: noop,
    onTrackpadScrollEnd: noop,
    revealWindow: () => native("reveal_window"),
    getSystemLocale: () => init.locale ?? null,
    getPreferredSystemLanguages: () => (init.locale ? [init.locale] : []),
    getPrimaryPasteEnabled: () => true,
    getLocalEnvironmentBootstraps: () => (localEnabled ? init.bootstraps || [] : []),
    getLocalEnvironmentEnabled: () => localEnabled,
    setLocalEnvironmentEnabled: async (enabled) => {
      await host("cc3:set-local-environment-enabled", enabled);
      localEnabled = enabled;
      location.reload();
    },
    getHostEnvironmentId: async () => null,
    getLocalEnvironmentBearerToken: getBearer,
    getClientSettings: () => host("desktop:get-client-settings"),
    setClientSettings: (settings) => host("desktop:set-client-settings", settings),
    getConnectionCatalog: () => host("desktop:get-connection-catalog"),
    setConnectionCatalog: (catalog) => host("desktop:set-connection-catalog", catalog),
    discoverSshHosts: () => host("desktop:discover-ssh-hosts"),
    resolveSshHost: (alias) => host("desktop:resolve-ssh-host", alias),
    requestSnapShotPermissions: (includeAccessibility) =>
      host("desktop:request-snap-shot-permissions", includeAccessibility),
    getSnapShotState: () => host("desktop:get-snap-shot-state"),
    setupSnapShot: (action) => host("desktop:setup-snap-shot", action),
    previewSnapShotConfig: (request) => host("desktop:preview-snap-shot-config", request),
    applySnapShotConfig: (previewId) => host("desktop:apply-snap-shot-config", previewId),
    checkSnapShotShortcut: (shortcut) => host("desktop:check-snap-shot-shortcut", shortcut),
    setSnapShotShortcutSuppressed: (suppressed) =>
      host("desktop:set-snap-shot-shortcut-suppressed", suppressed),
    listPendingSnapShots: () => host("desktop:list-pending-snap-shots"),
    readSnapShot: (id) => host("desktop:read-snap-shot", id),
    setSnapShotAnimationDestination: (destination) =>
      host("desktop:set-snap-shot-animation-destination", destination),
    dismissSnapShotAnimation: (id) => host("desktop:dismiss-snap-shot-animation", id),
    acknowledgeSnapShot: (id) => host("desktop:acknowledge-snap-shot", id),
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
    getServerExposureState: () => host("desktop:get-server-exposure-state"),
    setServerExposureMode: (mode) => host("desktop:set-server-exposure-mode", mode),
    setTailscaleServeEnabled: (input) => host("desktop:set-tailscale-serve-enabled", input),
    getAdvertisedEndpoints: () => host("desktop:get-advertised-endpoints"),
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
    receiveProviderAuthCallback: (url) => host("desktop:receive-provider-auth-callback", url),
    cancelProviderAuthCallback: (url) => host("desktop:cancel-provider-auth-callback", url),
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
    onSnapShotEvent: (fn) => on("desktop:snap-shot-event", fn),
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
      getState: () => host("desktop:cli-command-get-state"),
      install: () => host("desktop:cli-command-install"),
      uninstall: () => host("desktop:cli-command-uninstall"),
      addToPath: () => host("desktop:cli-command-add-to-path"),
    },
    webLinks: {
      setReady: (ready) => native("web_links_ready", ready),
      onOpen: (fn) => on("web-link-open", fn),
    },
    appActivation: {
      setReady: async () => {},
      complete: async () => {},
      onRequest: (fn) => on("app-activation", fn),
    },
    // `preview` is intentionally absent: the UI then uses the server-hosted browser (screencast).
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
    const bridge = window.desktopBridge;
    const report = (label) => (r) => window.__cc3Log?.(`selftest ${label} ${JSON.stringify(r)}`);
    bridge.webLinks.onOpen((url) => window.__cc3Log?.(`selftest web-link ${url}`));
    bridge.onSnapShotEvent((e) => window.__cc3Log?.(`selftest snap-event ${JSON.stringify(e)}`));
    bridge.webLinks.setReady(true);
    setTimeout(() => {
      window.__cc3Log?.("selftest notify " + typeof Notification + " " + Notification.permission);
      const n = new Notification("Input needed", { body: "Ping Check, No Task Yet" });
      n.addEventListener("click", () => window.__cc3Log?.("selftest notification-click"));
    }, 8000);
    setTimeout(async () => {
      await bridge.getSnapShotState().then(report("snapState"), report("snapState failed"));
      await bridge.cliCommand.getState().then(report("cli"), report("cli failed"));
      await bridge
        .getClientSettings()
        .then(report("clientSettings"), report("clientSettings failed"));
      await bridge
        .setServerExposureMode("network-accessible")
        .then(report("exposure"), report("exposure failed"));
      setTimeout(async () => {
        const res = await fetch("/").then((r) => r.status, String);
        window.__cc3Log?.("selftest after-restart fetch " + res);
        await bridge.getAdvertisedEndpoints().then(report("endpoints"), report("endpoints failed"));
        await bridge
          .setServerExposureMode("local-only")
          .then(report("exposure-back"), report("exposure-back failed"));
      }, 15000);
    }, 3000);
    // The host exits when idle (host/main.ts); a call after that must start it again.
    setTimeout(() => {
      bridge
        .getClientSettings()
        .then(report("late clientSettings"), report("late clientSettings failed"));
    }, 110000);
  }

  let localEnabled = init.localEnvironmentEnabled !== false;
})();
