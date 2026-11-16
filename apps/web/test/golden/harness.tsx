import { createMemoryHistory } from "@tanstack/react-router";
import { DEFAULT_CLIENT_SETTINGS } from "@t3tools/contracts/settings";
import { expect } from "vite-plus/test";
import { commands, page } from "vite-plus/test/browser";

import "~/index.css";

import { AppRoot } from "~/AppRoot";
import { __setClientSettingsForTests } from "~/hooks/useSettings";
import { getRouter } from "~/router";
import { fakeTimers, render } from "../render";

const FREEZE_CSS = `*,*::before,*::after{animation-duration:0s!important;animation-delay:0s!important;transition-duration:0s!important;transition-delay:0s!important;caret-color:transparent!important}`;
const compare = (allowedMismatchedPixels: number) =>
  ({
    comparatorName: "pixelmatch",
    comparatorOptions: { threshold: 0, allowedMismatchedPixels },
  }) as const;

declare module "vite-plus/test/browser" {
  interface BrowserCommands {
    domExportEnabled: () => Promise<boolean>;
    writeDomSnapshot: (name: string, html: string) => Promise<void>;
    writeGeometry: (name: string, json: string) => Promise<void>;
  }
}

/** Whole page as standalone HTML (DOM plus every stylesheet) for the native renderer. */
function standaloneHtml(): string {
  const css = [...document.styleSheets]
    .map((sheet) => {
      try {
        return [...sheet.cssRules].map((rule) => rule.cssText).join("\n");
      } catch {
        return "";
      }
    })
    .join("\n");
  const root = document.documentElement;
  return `<!doctype html><html class="${root.className}" style="${root.getAttribute("style") ?? ""}"><head><meta charset="utf-8"><style>${css}</style></head>${document.body.outerHTML}</html>`;
}

/**
 * Box of every layout-relevant element in document order: [tag, x, y, width, height, display].
 * Descendants of <svg> are left out (they are not CSS boxes); the native side skips the same ones.
 */
function geometry(): string {
  const boxes: Array<[string, number, number, number, number, string]> = [];
  for (const element of document.body.querySelectorAll("*")) {
    if (element.closest("svg") !== null && element.tagName.toLowerCase() !== "svg") continue;
    const r = element.getBoundingClientRect();
    boxes.push([
      element.tagName.toLowerCase(),
      r.x,
      r.y,
      r.width,
      r.height,
      getComputedStyle(element).display,
    ]);
  }
  return JSON.stringify(boxes);
}

export async function setupGolden() {
  await page.viewport(1400, 900);
  __setClientSettingsForTests({
    ...DEFAULT_CLIENT_SETTINGS,
    onboardingCompletedAt: "2026-01-01T00:00:00.000Z",
  });
}

export interface GoldenApp extends AsyncDisposable {
  readonly container: HTMLElement;
  /** `tolerance` is a pixel budget for states with known sub-pixel jitter; the default is 0. */
  readonly snap: (name: string, tolerance?: number) => Promise<void>;
  readonly settle: () => Promise<void>;
  /** Plays the animation a trigger starts and captures it at fixed times (ms). */
  readonly frames: (
    name: string,
    trigger: () => Promise<void>,
    times?: ReadonlyArray<number>,
  ) => Promise<void>;
}

/** Mounts the whole app in one theme. `animated` leaves CSS animations on for `frames`. */
export async function mountGolden(
  path: string,
  theme: "light" | "dark",
  { animated = false }: { animated?: boolean } = {},
): Promise<GoldenApp> {
  const clock = fakeTimers({ toFake: ["Date"], now: new Date("2026-03-18T09:00:30.000Z") });
  window.localStorage.setItem("t3code:theme", theme);
  const style = document.createElement("style");
  style.textContent = FREEZE_CSS;
  if (!animated) document.head.append(style);
  const router = getRouter(createMemoryHistory({ initialEntries: [path] }));
  await router.load();
  const container = document.createElement("div");
  container.style.cssText = "height: 900px; width: 1400px;";
  const screen = await render(<AppRoot router={router} />, { container, router: false });
  const settle = async () => {
    await document.fonts.ready;
    await new Promise((resolve) => setTimeout(resolve, 600));
  };
  const capture = (name: string, tolerance = 0) =>
    expect(page.elementLocator(container)).toMatchScreenshot(
      `${name}-${theme}`,
      compare(tolerance),
    );
  return {
    container,
    settle,
    snap: async (name, tolerance) => {
      await settle();
      if (await commands.domExportEnabled()) {
        await commands.writeDomSnapshot(`${name}-${theme}`, standaloneHtml());
        await commands.writeGeometry(`${name}-${theme}`, geometry());
      }
      await capture(name, tolerance);
    },
    frames: async (name, trigger, times = [0, 60, 120, 200, 400]) => {
      await settle();
      await trigger();
      for (const time of times) {
        // Park every running animation at an exact time so frames do not depend on wall clock.
        for (const animation of document.getAnimations()) {
          animation.pause();
          animation.currentTime = time;
        }
        await new Promise((resolve) => requestAnimationFrame(() => resolve(undefined)));
        await capture(`${name}-t${String(time).padStart(4, "0")}`);
      }
    },
    async [Symbol.asyncDispose]() {
      try {
        await screen[Symbol.asyncDispose]();
      } finally {
        style.remove();
        clock[Symbol.dispose]();
      }
    },
  };
}
