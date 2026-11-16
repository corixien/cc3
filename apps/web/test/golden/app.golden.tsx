// Golden screenshots of whole routes against the in-process fake environment.
// Record:  vp test run --project golden -u      Compare: vp test run --project golden
import type { ThreadId } from "@t3tools/contracts";
import { expect, it } from "vite-plus/test";
import { page } from "vite-plus/test/browser";

import {
  ENVIRONMENT_ID,
  installFakeEnvironment,
  threadWithHistory,
  threadWithQuestion,
} from "../fakeEnvironment";
import { mountGolden, setupGolden } from "./harness";

const HISTORY = threadWithHistory("thread-golden", 6);
const QUESTION = threadWithQuestion("thread-golden-question", "Which approach should I take?");
installFakeEnvironment([HISTORY, QUESTION]);
await setupGolden();

const threadPath = (id: ThreadId) => `/${ENVIRONMENT_ID}/${id}`;

async function shot(
  name: string,
  path: string,
  theme: "light" | "dark",
  ready: () => Promise<void>,
) {
  await using app = await mountGolden(path, theme);
  await ready();
  await app.snap(name);
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
  for (const route of ["usage", "pull-requests", "welcome"]) {
    it(`route ${route} ${theme}`, async () => {
      await shot(`route-${route}`, `/${route}`, theme, async () => {});
    });
  }
  for (const name of settingsPages) {
    it(`settings ${name} ${theme}`, async () => {
      await shot(`settings-${name}`, `/settings/${name}`, theme, async () => {});
    });
  }
}
