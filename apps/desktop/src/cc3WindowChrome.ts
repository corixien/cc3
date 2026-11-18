// CC3: renderer half of the custom Linux window controls. The frameless, transparent window gets
// a rounded 1px outline, caption buttons at the top right and a Window Controls Overlay polyfill
// so the web UI lays out its titlebar exactly as it does with native controls.
import { ipcRenderer, webFrame } from "electron";

import type { Cc3WindowOp, Cc3WindowState } from "./cc3WindowControls.ts";

const CHANNEL = "cc3:window";
const CONTROLS_WIDTH = 138;
const TITLEBAR_HEIGHT = 40;
const RADIUS = 14;

const call = (op: Cc3WindowOp): Promise<Cc3WindowState> => ipcRenderer.invoke(CHANNEL, op);

const ICONS = {
  min: '<path d="M0 5.5h10"/>',
  max: '<rect x="0.5" y="0.5" width="9" height="9"/>',
  restore: '<path d="M2.5 2.5v-2h7v7h-2M0.5 2.5h7v7h-7z"/>',
  close: '<path d="M0 0l10 10M10 0L0 10"/>',
};

const CSS = `
  .wco {
    --native-titlebar-height: ${TITLEBAR_HEIGHT}px;
    --workspace-topbar-height: ${TITLEBAR_HEIGHT}px;
    --workspace-controls-top: 0px;
    --workspace-controls-left: 0.75rem;
    --workspace-controls-right: calc(${CONTROLS_WIDTH}px + 0.75rem);
    --workspace-native-controls-inset: calc(${CONTROLS_WIDTH}px + 0.75rem);
  }
  /* A near-transparent html background stops body's background from propagating to the canvas,
     which would paint the window corners; body is rounded and clipped instead. */
  html { background-color: rgba(0, 0, 0, 0.004) !important; }
  body { border-radius: ${RADIUS}px; overflow: hidden; clip-path: inset(0 round ${RADIUS}px); }
  html.cc3-square body { border-radius: 0; clip-path: none; }
  html::after { content: ""; position: fixed; inset: 0; border: 1px solid #252525;
    border-radius: ${RADIUS}px; pointer-events: none; z-index: 2147483646; }
  html.cc3-square::after { display: none; }
  #cc3-controls { position: fixed; top: 0; right: 0; z-index: 2147483647; display: flex;
    height: ${TITLEBAR_HEIGHT}px; color: #f8fafc; user-select: none; -webkit-app-region: no-drag; }
  #cc3-controls.light { color: #1f2937; }
  #cc3-controls button { all: unset; box-sizing: border-box; width: 46px; height: 100%;
    display: flex; align-items: center; justify-content: center; cursor: default;
    -webkit-app-region: no-drag; }
  #cc3-controls button:hover { background: rgba(127,127,127,.22); }
  #cc3-controls button.close:hover { background: #e81123; color: #fff; }
  #cc3-controls svg { width: 10px; height: 10px; stroke: currentColor; fill: none; stroke-width: 1; }
`;

function installChrome(): void {
  const style = document.createElement("style");
  style.textContent = CSS;
  document.documentElement.append(style);

  const root = document.documentElement;
  const bar = document.createElement("div");
  bar.id = "cc3-controls";
  const button = (cls: keyof typeof ICONS, label: string, op: Cc3WindowOp) => {
    const b = document.createElement("button");
    b.className = cls;
    b.title = label;
    b.innerHTML = `<svg viewBox="0 0 10 10">${ICONS[cls]}</svg>`;
    b.addEventListener("click", () => void call(op).then(apply));
    return b;
  };
  const maxButton = button("max", "Maximize", "toggle-maximize");
  bar.append(button("min", "Minimize", "minimize"), maxButton, button("close", "Close", "close"));
  document.body.append(bar);

  function apply(state: Cc3WindowState): void {
    maxButton.innerHTML = `<svg viewBox="0 0 10 10">${state.maximized ? ICONS.restore : ICONS.max}</svg>`;
    root.classList.toggle("cc3-square", state.maximized || state.fullscreen);
    bar.style.display = state.fullscreen ? "none" : "flex";
  }
  const refresh = () => void call("state").then(apply);
  window.addEventListener("resize", refresh);
  refresh();

  const syncTheme = () => bar.classList.toggle("light", !root.classList.contains("dark"));
  new MutationObserver(syncTheme).observe(root, {
    attributes: true,
    attributeFilter: ["class"],
  });
  syncTheme();
}

export function installCc3WindowChrome(): void {
  // The overlay API lives in the page's world, so the polyfill runs there before app scripts.
  void webFrame.executeJavaScript(`(() => {
    const target = new EventTarget();
    const rect = () => new DOMRect(0, 0, Math.max(0, innerWidth - ${CONTROLS_WIDTH}), ${TITLEBAR_HEIGHT});
    Object.defineProperty(navigator, "windowControlsOverlay", {
      configurable: true,
      value: {
        visible: true,
        getTitlebarAreaRect: rect,
        addEventListener: (type, fn) => target.addEventListener(type, fn),
        removeEventListener: (type, fn) => target.removeEventListener(type, fn),
      },
    });
    addEventListener("resize", () => target.dispatchEvent(new Event("geometrychange")));
  })()`);
  if (document.readyState === "loading") {
    window.addEventListener("DOMContentLoaded", installChrome, { once: true });
  } else installChrome();
}
