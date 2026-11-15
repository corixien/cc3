// Resolves `electron` to an inert module so apps/desktop sources load in plain Node
// (the real package downloads a binary on import). Any use at runtime throws.
const names = [
  "app",
  "BrowserWindow",
  "dialog",
  "webContents",
  "ipcMain",
  "Menu",
  "MenuItem",
  "shell",
  "session",
  "safeStorage",
  "nativeTheme",
  "powerMonitor",
  "protocol",
  "screen",
  "clipboard",
  "Notification",
  "net",
  "WebContentsView",
  "nativeImage",
  "systemPreferences",
  "autoUpdater",
  "contextBridge",
  "ipcRenderer",
  "desktopCapturer",
  "globalShortcut",
  "utilityProcess",
  "MessageChannelMain",
  "powerSaveBlocker",
  "Tray",
  "crashReporter",
  "webUtils",
  "webFrame",
  "View",
  "BaseWindow",
];
const source =
  `const inert = (name) => new Proxy(function () {}, { get: (_, key) => key === "then" ? undefined : inert(name + "." + String(key)), apply() { throw new Error("electron." + name + " is not available in the CC3 host"); }, construct() { throw new Error("electron." + name + " is not available in the CC3 host"); } });\n` +
  names.map((n) => `export const ${n} = inert(${JSON.stringify(n)});`).join("\n") +
  "\nexport default { " +
  names.join(", ") +
  " };";

export async function resolve(specifier, context, next) {
  if (specifier === "electron") {
    return { url: `data:text/javascript,${encodeURIComponent(source)}`, shortCircuit: true };
  }
  return next(specifier, context);
}
