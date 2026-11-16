// Golden screenshots of the whole app against the in-process fake environment.
// Record:  vp test run --project golden -u      Compare: vp test run --project golden
// Same harness as test/appScenarios.renders.tsx; every state is mounted at 1400x900 with a
// frozen clock and with CSS animations and transitions parked at their end state.
import { createMemoryHistory } from "@tanstack/react-router";
import { DEFAULT_CLIENT_SETTINGS } from "@t3tools/contracts/settings";
import type { ThreadId } from "@t3tools/contracts";
import type * as Context from "effect/Context";
import { expect, it, vi } from "vite-plus/test";
import { page } from "vite-plus/test/browser";

import "~/index.css";

import { AppRoot } from "~/AppRoot";
import { __setClientSettingsForTests } from "~/hooks/useSettings";
import { getRouter } from "~/router";
import {
  ENVIRONMENT_ID,
  installFakeEnvironment,
  threadWithHistory,
  threadWithQuestion,
  type FakeDriver,
} from "../fakeEnvironment";
import { fakeTimers, render } from "../render";

vi.mock("../../../../packages/client-runtime/src/connection/driver.ts", async (importOriginal) => {
  const [actual, { fakeDriver }, Layer] = await Promise.all([
    importOriginal<{ readonly ConnectionDriver: Context.Key<unknown, FakeDriver> }>(),
    import("../fakeEnvironment"),
    import("effect/Layer"),
  ]);
  return { ...actual, layer: Layer.succeed(actual.ConnectionDriver, fakeDriver) };
});
vi.mock(import("~/rpc/requestLatencyState"), async (original) => ({
  ...(await original()),
  trackRpcRequestSent: () => {},
}));

const HISTORY = threadWithHistory("thread-golden", 6);
const QUESTION = threadWithQuestion("thread-golden-question", "Which approach should I take?");
installFakeEnvironment([HISTORY, QUESTION]);
await page.viewport(1400, 900);
__setClientSettingsForTests({
  ...DEFAULT_CLIENT_SETTINGS,
  onboardingCompletedAt: "2026-01-01T00:00:00.000Z",
});

const FREEZE_CSS = `*,*::before,*::after{animation-duration:0s!important;animation-delay:0s!important;transition-duration:0s!important;transition-delay:0s!important;caret-color:transparent!important}`;
const threadPath = (id: ThreadId) => `/${ENVIRONMENT_ID}/${id}`;

async function shot(
  name: string,
  path: string,
  theme: "light" | "dark",
  ready: () => Promise<void>,
) {
  using _clock = fakeTimers({ toFake: ["Date"], now: new Date("2026-03-18T09:00:30.000Z") });
  window.localStorage.setItem("t3code:theme", theme);
  const style = document.createElement("style");
  style.textContent = FREEZE_CSS;
  document.head.append(style);
  const router = getRouter(createMemoryHistory({ initialEntries: [path] }));
  await router.load();
  const container = document.createElement("div");
  container.style.cssText = "height: 900px; width: 1400px;";
  await using _app = await render(<AppRoot router={router} />, { container, router: false });
  try {
    await ready();
    await document.fonts.ready;
    await new Promise((resolve) => setTimeout(resolve, 600));
    await expect(page.elementLocator(container)).toMatchScreenshot(`${name}-${theme}`, {
      comparatorName: "pixelmatch",
      comparatorOptions: { threshold: 0, allowedMismatchedPixels: 0 },
    });
  } finally {
    style.remove();
  }
}

const settingsPages = Object.keys(import.meta.glob("../../src/routes/_chat.settings.*.tsx"))
  .map((file) => /_chat\.settings\.(.+)\.tsx$/.exec(file)?.[1])
  .filter((name): name is string => Boolean(name));

for (const theme of ["dark", "light"] as const) {
  it(`thread history ${theme}`, async () => {
    await shot("thread-history", threadPath(HISTORY.id), theme, () =>
      expect.element(page.getByText("Answer 5 for thread-golden.")).toBeVisible(),
    );
  });
  it(`thread question ${theme}`, async () => {
    await shot("thread-question", threadPath(QUESTION.id), theme, () =>
      expect.element(page.getByText("Which approach should I take?")).toBeVisible(),
    );
  });
  it(`new chat ${theme}`, async () => {
    await shot("new-chat", "/new", theme, async () => {});
  });
  for (const name of settingsPages) {
    it(`settings ${name} ${theme}`, async () => {
      await shot(`settings-${name}`, `/settings/${name}`, theme, async () => {});
    });
  }
}
