#!/usr/bin/env node
// Erzeugt docs/perf/ANIMATION-CATALOG.md aus dem Web-Quellcode: @keyframes, CSS-Variablen fuer
// Easing/Dauer und alle animate-/transition-/duration-/ease-Klassen je Datei.
import * as NodeFS from "node:fs";
import * as NodePath from "node:path";

const root = new URL("../../../../apps/web/src/", import.meta.url).pathname;
const files = [];
const walk = (dir) => {
  for (const entry of NodeFS.readdirSync(dir, { withFileTypes: true })) {
    const full = NodePath.join(dir, entry.name);
    if (entry.isDirectory()) walk(full);
    else if (/\.(tsx?|css)$/.test(entry.name) && !/\.test\./.test(entry.name)) files.push(full);
  }
};
walk(root);

const keyframes = [];
const usage = new Map();
const springs = [];
const classRe =
  /\b((?:[a-z-]+:)*(?:animate-[\w\[\]\.\-\/]+|transition(?:-[\w\[\]\.\-\/]+)?|duration-[\w\[\]\.]+|ease-[\w\[\]\.\(\),\-]+|delay-[\w\[\]\.]+))/g;
for (const file of files) {
  const text = NodeFS.readFileSync(file, "utf8");
  const rel = NodePath.relative(root, file);
  for (const m of text.matchAll(/@keyframes\s+([\w-]+)\s*\{([\s\S]*?\n\})/g)) {
    keyframes.push({ rel, name: m[1], body: m[2].replace(/\s+/g, " ").trim().slice(0, 220) });
  }
  const found = new Set([...text.matchAll(classRe)].map((m) => m[1]));
  if (found.size) usage.set(rel, [...found].sort());
  for (const m of text.matchAll(
    /cubic-bezier\([^)]*\)|spring\([^)]*\)|(?:stiffness|damping|duration):\s*[\d.]+/g,
  )) {
    springs.push(`${rel}: ${m[0]}`);
  }
}

const lines = [
  "# Animationskatalog (automatisch erzeugt)",
  "",
  "Erzeugt von `apps/cc3-desktop/scripts/perf/extract-animations.mjs`. Quelle fuer Phase 1.3 und 7 (native Nachbildung).",
  "Fuer jede Animation muss eine Bildfolge bei 0/25/50/75/100 Prozent gegen die Referenz verglichen werden (siehe PLAN, Phase 1.3).",
  "",
  `## @keyframes (${keyframes.length})`,
  "",
  "| Datei | Name | Inhalt |",
  "|---|---|---|",
  ...keyframes.map((k) => `| ${k.rel} | ${k.name} | \`${k.body.replace(/\|/g, "/")}\` |`),
  "",
  "## Zeitfunktionen und Federn im Code",
  "",
  ...[...new Set(springs)].sort().map((s) => `- ${s}`),
  "",
  `## Tailwind-Klassen fuer Animation/Transition je Datei (${usage.size} Dateien)`,
  "",
  ...[...usage]
    .sort()
    .flatMap(([rel, cls]) => [`- \`${rel}\`: ${cls.map((c) => `\`${c}\``).join(" ")}`]),
  "",
];
const out = new URL("../../../../docs/perf/ANIMATION-CATALOG.md", import.meta.url).pathname;
NodeFS.writeFileSync(out, lines.join("\n"));
console.log(`keyframes=${keyframes.length} files=${usage.size} -> ${out}`);
