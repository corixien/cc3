// Bundled stand-in for `electron` (see electron-stub.mjs for the dev loader hook).
const inert = (name) =>
  new Proxy(function () {}, {
    get: (_, key) => (key === "then" ? undefined : inert(`${name}.${String(key)}`)),
    apply() {
      throw new Error(`electron.${name} is not available in the CC3 host`);
    },
    construct() {
      throw new Error(`electron.${name} is not available in the CC3 host`);
    },
  });

export const app = inert("app");
export const BrowserWindow = inert("BrowserWindow");
export const dialog = inert("dialog");
export const webContents = inert("webContents");
export const ipcMain = inert("ipcMain");
export const Menu = inert("Menu");
export const shell = inert("shell");
export const session = inert("session");
export const safeStorage = inert("safeStorage");
export const nativeTheme = inert("nativeTheme");
export const powerMonitor = inert("powerMonitor");
export const protocol = inert("protocol");
export const screen = inert("screen");
export const clipboard = inert("clipboard");
export const Notification = inert("Notification");
export const net = inert("net");
export const WebContentsView = inert("WebContentsView");
export const nativeImage = inert("nativeImage");
export const systemPreferences = inert("systemPreferences");
export const autoUpdater = inert("autoUpdater");
export const contextBridge = inert("contextBridge");
export const ipcRenderer = inert("ipcRenderer");
export const desktopCapturer = inert("desktopCapturer");
export const globalShortcut = inert("globalShortcut");
export const utilityProcess = inert("utilityProcess");
export const MessageChannelMain = inert("MessageChannelMain");
export const ClipboardItem = inert("ClipboardItem");
export default {};
