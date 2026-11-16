// Exports whole-app DOM + CSS as standalone HTML for the native renderer spike (apps/cc3-native).
import { expect, it } from "vite-plus/test";
import { commands, page } from "vite-plus/test/browser";

import { ENVIRONMENT_ID, installFakeEnvironment, threadWithHistory } from "../fakeEnvironment";
import { mountGolden, setupGolden } from "./harness";

declare module "vite-plus/test/browser" {
  interface BrowserCommands {
    writeDomSnapshot: (name: string, html: string) => Promise<void>;
  }
}

const THREAD = threadWithHistory("thread-golden", 6);
installFakeEnvironment([THREAD]);
await setupGolden();

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

it("export thread history", async () => {
  await using app = await mountGolden(`/${ENVIRONMENT_ID}/${THREAD.id}`, "dark");
  await expect.element(page.getByText("Answer 5 for thread-golden.")).toBeVisible();
  await app.settle();
  await commands.writeDomSnapshot("thread-history-dark", standaloneHtml());
});
