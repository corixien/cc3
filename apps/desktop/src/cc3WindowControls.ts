// CC3: main-process half of the custom Linux window controls (see cc3WindowChrome.ts).
import { BrowserWindow, ipcMain } from "electron";

export const CC3_WINDOW_CHANNEL = "cc3:window";
export type Cc3WindowOp = "state" | "minimize" | "toggle-maximize" | "close";
export interface Cc3WindowState {
  readonly maximized: boolean;
  readonly fullscreen: boolean;
}

ipcMain.handle(CC3_WINDOW_CHANNEL, (event, op: Cc3WindowOp): Cc3WindowState => {
  const window = BrowserWindow.fromWebContents(event.sender);
  if (!window || window.isDestroyed()) return { maximized: false, fullscreen: false };
  if (op === "minimize") window.minimize();
  else if (op === "close") window.close();
  else if (op === "toggle-maximize") {
    if (window.isMaximized()) window.unmaximize();
    else window.maximize();
  }
  return { maximized: window.isMaximized(), fullscreen: window.isFullScreen() };
});
