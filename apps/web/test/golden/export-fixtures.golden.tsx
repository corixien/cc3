// Writes the golden fixture data (shell and thread projection) as JSON for the native app's tests.
import { it } from "vite-plus/test";
import { commands } from "vite-plus/test/browser";

import {
  projectionOf,
  shellOf,
  threadWithHistory,
  threadWithQuestion,
  threadWithWork,
} from "../fakeEnvironment";

declare module "vite-plus/test/browser" {
  interface BrowserCommands {
    writeFixture: (name: string, json: string) => Promise<void>;
  }
}

it("export fixtures", async () => {
  const history = threadWithHistory("thread-golden", 6);
  const question = threadWithQuestion("thread-golden-question", "Which approach should I take?");
  const work = threadWithWork("thread-golden-work");
  for (const thread of [history, question, work]) {
    await commands.writeFixture(
      `${thread.id}.projection`,
      JSON.stringify(projectionOf(thread), null, 1),
    );
    await commands.writeFixture(`${thread.id}.shell`, JSON.stringify(shellOf(thread), null, 1));
  }
});
