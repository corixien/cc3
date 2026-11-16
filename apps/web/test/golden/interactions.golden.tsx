// Golden screenshots of interaction states (overlays, hover, focus) and of animation frames.
import { expect, it } from "vite-plus/test";
import { page, userEvent } from "vite-plus/test/browser";

import { ENVIRONMENT_ID, installFakeEnvironment, threadWithHistory } from "../fakeEnvironment";
import { toastManager } from "~/components/ui/toast";
import { dispatchCommandPalette } from "~/commandPaletteStore";
import { act } from "../render";
import { mountGolden, setupGolden } from "./harness";

const THREAD = threadWithHistory("thread-golden", 6);
const OTHER = threadWithHistory("thread-other", 3);
installFakeEnvironment([THREAD, OTHER]);
await setupGolden();
const path = `/${ENVIRONMENT_ID}/${THREAD.id}`;
const ready = () => expect.element(page.getByText("Answer 5 for thread-golden.")).toBeVisible();
const key = (combo: string) => userEvent.keyboard(combo);
const openPalette = () => act(async () => dispatchCommandPalette({ _tag: "SetOpen", open: true }));
const closePalette = () =>
  act(async () => dispatchCommandPalette({ _tag: "SetOpen", open: false }));
// Panels keep their state across tests in this file, so each test that toggles one undoes it.
const toggle = (combo: string) =>
  ({ [Symbol.asyncDispose]: () => key(combo) }) satisfies AsyncDisposable;

for (const theme of ["dark", "light"] as const) {
  it(`command palette ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    await openPalette();
    await expect.element(page.getByLabelText("Command palette")).toBeVisible();
    await app.snap("overlay-command-palette");
    await closePalette();
  });
  it(`command palette query ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    await openPalette();
    await expect.element(page.getByLabelText("Command palette")).toBeVisible();
    await key("thread");
    // Message search results arrive asynchronously; wait until they stop changing.
    await new Promise((resolve) => setTimeout(resolve, 2000));
    await app.snap("overlay-command-palette-query", 16);
    await closePalette();
  });
  it(`diff panel ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    await using _undo = toggle("{Control>}d{/Control}");
    await key("{Control>}d{/Control}");
    await expect.element(page.getByRole("group", { name: "Diff layout" })).toBeVisible();
    await app.snap("panel-diff");
  });
  it(`terminal drawer ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    await using _undo = toggle("{Control>}j{/Control}");
    await key("{Control>}j{/Control}");
    await app.snap("panel-terminal");
  });
  it(`sidebar collapsed ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    await using _undo = toggle("{Control>}b{/Control}");
    await key("{Control>}b{/Control}");
    await app.snap("sidebar-collapsed");
  });
  it(`sidebar row hover ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    await page.getByText("Thread thread-other").hover({ force: true });
    await app.snap("sidebar-row-hover");
  });
  it(`sidebar row context menu ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    await page.getByText("Thread thread-other").click({ button: "right", force: true });
    await app.snap("sidebar-row-context-menu");
  });
  it(`composer typed ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    const editor = page.getByTestId("composer-editor");
    await userEvent.click(editor);
    await userEvent.type(editor, "fix the flaky tests!");
    await app.snap("composer-typed");
    // The draft survives the test; clear it so later states start from an empty composer.
    await userEvent.click(editor);
    await key("{Control>}a{/Control}{Backspace}");
  });
  it(`model picker ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    await page.getByRole("button", { name: /GPT-5\.4/ }).click();
    await app.snap("composer-model-picker");
  });
  it(`toast ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    toastManager.add({
      type: "success",
      title: "Golden toast",
      description: "Shown for screenshots",
    });
    await expect.element(page.getByText("Golden toast")).toBeVisible();
    await app.snap("toast-success");
  });
  it(`header tooltip ${theme}`, async () => {
    await using app = await mountGolden(path, theme);
    await ready();
    await page.getByLabelText("Toggle terminal drawer").hover({ force: true });
    await app.snap("tooltip-header-terminal");
  });
}

// Animation frames: the same trigger is captured at fixed times; see harness.frames.
for (const theme of ["dark", "light"] as const) {
  it(`frames command palette ${theme}`, async () => {
    await using app = await mountGolden(path, theme, { animated: true });
    await ready();
    await app.frames("anim-command-palette", openPalette);
    await closePalette();
  });
  it(`frames diff panel ${theme}`, async () => {
    await using app = await mountGolden(path, theme, { animated: true });
    await ready();
    await using _undo = toggle("{Control>}d{/Control}");
    await app.frames("anim-diff-panel", () => key("{Control>}d{/Control}"));
  });
  it(`frames sidebar toggle ${theme}`, async () => {
    await using app = await mountGolden(path, theme, { animated: true });
    await ready();
    await using _undo = toggle("{Control>}b{/Control}");
    await app.frames("anim-sidebar-toggle", () => key("{Control>}b{/Control}"));
  });
  it(`frames toast ${theme}`, async () => {
    await using app = await mountGolden(path, theme, { animated: true });
    await ready();
    await app.frames("anim-toast", async () => {
      toastManager.add({
        type: "success",
        title: "Golden toast",
        description: "Shown for screenshots",
      });
    });
  });
  it(`frames model picker ${theme}`, async () => {
    await using app = await mountGolden(path, theme, { animated: true });
    await ready();
    await app.frames("anim-model-picker", () =>
      page.getByRole("button", { name: /GPT-5\.4/ }).click(),
    );
  });
}
