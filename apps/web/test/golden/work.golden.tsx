import type { ThreadId } from "@t3tools/contracts";
import { expect, it } from "vite-plus/test";
import { page } from "vite-plus/test/browser";

import { ENVIRONMENT_ID, installFakeEnvironment, threadWithWork } from "../fakeEnvironment";
import { mountGolden, setupGolden } from "./harness";

const WORK = threadWithWork("thread-golden-work");
installFakeEnvironment([WORK]);
await setupGolden();

for (const theme of ["dark", "light"] as const) {
  it(`thread work ${theme}`, async () => {
    await using app = await mountGolden(`/${ENVIRONMENT_ID}/${WORK.id as ThreadId}`, theme);
    await expect.element(page.getByText("Done.", { exact: false }).first()).toBeVisible();
    await app.snap("thread-work");
    await page.getByRole("button", { name: /^Worked for/ }).click();
    await expect.element(page.getByRole("button", { name: /^Ran 1 command/ })).toBeVisible();
    await app.snap("thread-work-expanded");
    if (theme === "dark") {
      await page.getByRole("button", { name: /^Ran 1 command/ }).click();
      await expect.element(page.getByText("pnpm", { exact: false }).first()).toBeVisible();
      await app.snap("thread-work-entries");
    }
  });
}
