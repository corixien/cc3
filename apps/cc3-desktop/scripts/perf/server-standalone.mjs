import * as NodeChildProcess from "node:child_process";
import * as NodeFS from "node:fs";
const R = "/home/mateo/.local/share/cc3-app/usr/lib/CC3/resources";
const port = 43999;
NodeFS.writeFileSync(
  "/tmp/cc3srv/boot.json",
  JSON.stringify({
    mode: "desktop",
    noBrowser: true,
    port,
    t3Home: "/tmp/cc3srv/home",
    host: "127.0.0.1",
    desktopBootstrapToken: "a".repeat(48),
    tailscaleServeEnabled: false,
    tailscaleServePort: 443,
    desktopAppUpdate: false,
  }) + "\n",
);
const c = NodeChildProcess.spawn(
  R + "/node",
  [
    "--require",
    "" + new URL("./server-mem-preload.cjs", import.meta.url).pathname + "",
    ...(process.argv[2] ? process.argv[2].split(" ") : []),
    R + "/app/apps/server/dist/bin.mjs",
    "--bootstrap-fd",
    "0",
  ],
  {
    stdio: [NodeFS.openSync("/tmp/cc3srv/boot.json", "r"), "ignore", "inherit"],
    env: { ...process.env, CC3_HOME: "/tmp/cc3srv/home" },
  },
);
console.log("pid", c.pid);
setTimeout(
  () => {
    c.kill("SIGTERM");
  },
  Number(process.env.RUNMS || 50000),
);
c.on("exit", () => process.exit(0));
