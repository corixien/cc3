// CC3: a chat finished while CC3 was in the background. A dimmed, always-on-top overlay over the screen
// shows it in the middle; a click on the card opens the chat, a click anywhere else dismisses it.
import * as Electron from "electron";

export const CC3_COMPLETION_OVERLAY_CHANNEL = "cc3:completion-overlay";

interface OverlayInput {
  readonly title: string;
  readonly message: string;
  readonly environmentId: string;
  readonly threadId: string;
}

const escapeHtml = (value: string) =>
  value.replace(/[&<>"']/g, (char) => `&#${char.charCodeAt(0)};`);

function overlayHtml(input: OverlayInput): string {
  return `<!doctype html><meta charset="utf-8"><style>
html,body{margin:0;height:100%;background:rgba(0,0,0,.55);font-family:system-ui,sans-serif;color:#e8e8e8;overflow:hidden;cursor:default}
.wrap{height:100%;display:flex;align-items:center;justify-content:center}
.card{min-width:360px;max-width:60vw;padding:36px 44px;border-radius:20px;background:#0b0b0b;border:1px solid #333;box-shadow:0 30px 90px rgba(0,0,0,.7);text-align:center;cursor:pointer}
.card:hover{border-color:#777}
h1{margin:0 0 10px;font-size:30px;font-weight:600;word-break:break-word}
p{margin:0;font-size:18px;color:#a8a8a8}
</style><div class="wrap" id="bg"><div class="card" id="card"><h1>${escapeHtml(input.title)}</h1><p>${escapeHtml(input.message)}</p></div></div>
<script>
const go = (target) => { location.href = "cc3-overlay://" + target; };
document.getElementById("card").addEventListener("click", (event) => { event.stopPropagation(); go("open"); });
document.getElementById("bg").addEventListener("click", () => go("dismiss"));
addEventListener("keydown", (event) => { if (event.key === "Escape") go("dismiss"); });
try {
  const ctx = new AudioContext();
  [660, 880].forEach((freq, i) => {
    const osc = ctx.createOscillator(); const gain = ctx.createGain();
    osc.frequency.value = freq; osc.connect(gain); gain.connect(ctx.destination);
    const t = ctx.currentTime + i * 0.18;
    gain.gain.setValueAtTime(0.0001, t); gain.gain.exponentialRampToValueAtTime(0.25, t + 0.02);
    gain.gain.exponentialRampToValueAtTime(0.0001, t + 0.3);
    osc.start(t); osc.stop(t + 0.32);
  });
} catch {}
</script>`;
}

let overlay: Electron.BrowserWindow | null = null;

function mainWindow(): Electron.BrowserWindow | undefined {
  return Electron.BrowserWindow.getAllWindows().find(
    (window) => !window.isDestroyed() && window !== overlay && window.webContents.getURL() !== "",
  );
}

function closeOverlay(): void {
  if (overlay && !overlay.isDestroyed()) overlay.destroy();
  overlay = null;
}

function showOverlay(input: OverlayInput): void {
  closeOverlay();
  const main = mainWindow();
  const display = main
    ? Electron.screen.getDisplayMatching(main.getBounds())
    : Electron.screen.getPrimaryDisplay();
  const window = new Electron.BrowserWindow({
    ...display.bounds,
    frame: false,
    transparent: true,
    alwaysOnTop: true,
    skipTaskbar: true,
    resizable: false,
    movable: false,
    hasShadow: false,
    show: false,
    webPreferences: { sandbox: true, autoplayPolicy: "no-user-gesture-required" },
  });
  overlay = window;
  window.setAlwaysOnTop(true, "screen-saver");
  window.webContents.on("will-navigate", (event, url) => {
    if (!url.startsWith("cc3-overlay://")) return;
    event.preventDefault();
    const open = url === "cc3-overlay://open";
    closeOverlay();
    const target = mainWindow();
    if (!open || !target) return;
    if (target.isMinimized()) target.restore();
    target.show();
    target.focus();
    void target.webContents.executeJavaScript(
      `window.dispatchEvent(new CustomEvent("cc3-open-thread", { detail: ${JSON.stringify({
        environmentId: input.environmentId,
        threadId: input.threadId,
      })} }))`,
    );
  });
  window.once("ready-to-show", () => window.showInactive());
  void window.loadURL(`data:text/html;charset=utf-8,${encodeURIComponent(overlayHtml(input))}`);
}

export function installCc3CompletionOverlay(): void {
  Electron.ipcMain.handle(CC3_COMPLETION_OVERLAY_CHANNEL, (_event, input: unknown) => {
    if (typeof input !== "object" || input === null) return;
    const { title, message, environmentId, threadId } = input as Record<string, unknown>;
    if (
      typeof title !== "string" ||
      typeof message !== "string" ||
      typeof environmentId !== "string" ||
      typeof threadId !== "string"
    ) {
      return;
    }
    // Only when CC3 itself is not the window the user is working in.
    if (Electron.BrowserWindow.getFocusedWindow() !== null) return;
    showOverlay({
      title: title.slice(0, 200),
      message: message.slice(0, 200),
      environmentId,
      threadId,
    });
  });
}
