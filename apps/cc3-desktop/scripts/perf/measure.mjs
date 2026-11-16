#!/usr/bin/env node
// Misst RAM (PSS) und CPU-Zeit des Prozessbaums unter einer Wurzel-PID.
// Nutzung: node measure.mjs <rootPid> [--seconds 60] [--out file.json]
// Liest nur /proc, beendet keine Prozesse.
import * as NodeFS from "node:fs";
import * as NodePath from "node:path";

const args = process.argv.slice(2);
const root = Number(args[0]);
const opt = (k, d) => (args.includes(k) ? args[args.indexOf(k) + 1] : d);
const seconds = Number(opt("--seconds", 60));
const out = opt("--out", null);
if (!root) {
  console.error("usage: measure.mjs <rootPid> [--seconds N] [--out file.json]");
  process.exit(2);
}

const read = (p) => {
  try {
    return NodeFS.readFileSync(p, "utf8");
  } catch {
    return null;
  }
};
const children = () => {
  const map = new Map();
  for (const d of NodeFS.readdirSync("/proc")) {
    if (!/^\d+$/.test(d)) continue;
    const st = read(`/proc/${d}/stat`);
    if (!st) continue;
    const rest = st.slice(st.lastIndexOf(")") + 2).split(" ");
    const ppid = Number(rest[1]);
    (map.get(ppid) ?? map.set(ppid, []).get(ppid)).push(Number(d));
  }
  return map;
};
const tree = () => {
  const map = children();
  const all = [];
  const walk = (p) => {
    all.push(p);
    (map.get(p) ?? []).forEach(walk);
  };
  walk(root);
  return all;
};
const pss = (pid) => {
  const t = read(`/proc/${pid}/smaps_rollup`);
  const m = t && /^Pss:\s+(\d+) kB/m.exec(t);
  return m ? Number(m[1]) / 1024 : 0;
};
const cpu = (pid) => {
  const st = read(`/proc/${pid}/stat`);
  if (!st) return 0;
  const f = st.slice(st.lastIndexOf(")") + 2).split(" ");
  return (Number(f[11]) + Number(f[12])) / 100; // utime+stime in s
};
const name = (pid) => {
  const c = (read(`/proc/${pid}/cmdline`) ?? "").split("\0").filter(Boolean);
  const exe = (c[0] ?? "").split("/").pop();
  const hint = c.find((a) => /host\/main|apps\/server|WebKit|bin\.mjs/.test(a)) ?? "";
  return `${exe} ${hint.split("/").slice(-3).join("/")}`.trim();
};
const group = (n) =>
  /WebKit/.test(n)
    ? "webview"
    : /host\/main/.test(n)
      ? "host"
      : /apps\/server|bin\.mjs/.test(n)
        ? "server"
        : /^cc3\b/.test(n)
          ? "shell"
          : "agent-children";
const sample = () => {
  const pids = tree();
  const procs = pids.map((p) => ({
    pid: p,
    name: name(p),
    group: group(name(p)),
    pssMb: +pss(p).toFixed(1),
    cpuS: cpu(p),
  }));
  return { total: +procs.reduce((a, b) => a + b.pssMb, 0).toFixed(1), procs };
};

const t0 = Date.now();
const first = sample();
const cpu0 = first.procs.reduce((a, b) => a + b.cpuS, 0);
await new Promise((r) => setTimeout(r, seconds * 1000));
const last = sample();
const cpu1 = last.procs.reduce((a, b) => a + b.cpuS, 0);
const result = {
  at: new Date().toISOString(),
  seconds,
  pssTotalMb: last.total,
  cpuPercentOverWindow: +(((cpu1 - cpu0) / ((Date.now() - t0) / 1000)) * 100).toFixed(2),
  procs: last.procs,
};
console.log(
  `PSS total ${result.pssTotalMb} MB, CPU ${result.cpuPercentOverWindow}% over ${seconds}s`,
);
const byGroup = {};
for (const p of result.procs) byGroup[p.group] = +((byGroup[p.group] ?? 0) + p.pssMb).toFixed(1);
result.byGroup = byGroup;
console.log("by group:", JSON.stringify(byGroup));
for (const p of result.procs.sort((a, b) => b.pssMb - a.pssMb))
  console.log(`${String(p.pssMb).padStart(7)} MB  ${p.pid}  ${p.name}`);
if (out) {
  NodeFS.mkdirSync(NodePath.dirname(out), { recursive: true });
  NodeFS.writeFileSync(out, JSON.stringify(result, null, 2));
}
