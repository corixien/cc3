# Server-Speicher (Phase 2.1)

Messung: frischer Server (gebuendelter Node aus der installierten CC3-App, temporaeres Home, 26 bis 50 s Leerlauf), `process.memoryUsage()` pro Thread.
Skripte: `apps/cc3-desktop/scripts/perf/server-standalone.mjs` (startet den Server wie `server.rs`), `server-mem-preload.cjs` (loggt Speicher je Thread), `measure.mjs` (PSS des Prozessbaums).

## Befunde

- Hauptthread: V8-Heap reserviert 230 MB bei nur 118 MB belegt (grosse Young Generation), external 41 MB.
- Sechs weitere Worker-Threads (je ca. 7 MB Heap, aber je eigene Isolate): SQLite-Reader/Writer-Worker (`packages/shared/src/nodeSqliteClient.ts`, aktiviert in `apps/server/src/persistence/Sqlite.ts`) plus Checkpoint-Worker.
- Das laufende CC3 hatte ca. 130 MB des Server-Prozesses ausgelagert (Swap), Rechner unter Speicherdruck (13 GB belegt, 14 GB Swap). Messungen am laufenden System sind deshalb unzuverlaessig; frische Standalone-Laeufe sind die Referenz.
- Im Prozessbaum der laufenden App hingen ausserdem Agent-Kindprozesse (`claude`, `node`, `bun`, MCP), die kein Server-Overhead sind und getrennt gezaehlt werden muessen (`measure.mjs` gruppiert jetzt in `byGroup`).
- (Korrigiert, siehe Nachtrag 2) Der Wert von 8 MB fuer den Node-Host stammte aus einer ausgelagerten Messung.

## Flags (Hauptthread RSS, fresh start)

| Flags                                              | RSS                         | heapTotal                            |
| -------------------------------------------------- | --------------------------- | ------------------------------------ |
| keine                                              | 451 MB                      | 228 MB                               |
| `--max-semi-space-size=1`                          | 346 MB                      | 111 MB                               |
| `--max-semi-space-size=1 --max-old-space-size=160` | 326 MB                      | 105 MB                               |
| `--max-semi-space-size=1 --max-old-space-size=96`  | Absturz (Heap belegt 99 MB) |                                      |
| `--max-semi-space-size=1 --lite-mode`              | 291 MB                      | 91 MB (langsamer, nicht uebernommen) |

Uebernommen: `--max-semi-space-size=1` in `server.rs` (-105 MB, kein Funktionsrisiko, etwas haeufigere kleine GCs, CPU noch zu messen).

## Offen

- Der RSS des Servers bleibt bei ca. 350 MB; belegter V8-Heap ist 99 MB, external 41 MB. Rest (ca. 200 MB) ist Node-Binary, Worker-Isolates und native Speicher. Naechster Schritt: Worker-Anzahl und `resourceLimits` pro Worker, Heap-Snapshot der Startphase (was belegt 99 MB direkt nach dem Start: Module, Provider, Schemas), lazy Laden der Provider.
- Der Vergleichswert "Electron-Server 56 bis 61 MB" ist zu pruefen: vermutlich andere Messmethode (PSS eines Teilprozesses). Nachmessen mit demselben Skript.

## Phase 2 Nachtrag: Heap-Aufschluesselung und verworfene Ideen

Sampling-Heap-Profil (30 s nach Start, `--max-semi-space-size=1`, 88 MB belegt):

| Verbraucher                                                            | MB         |
| ---------------------------------------------------------------------- | ---------- |
| `dist/Schema-*.mjs` (Effect-Schema-Aufbau aller Vertraege beim Laden)  | 31 bis 33  |
| native/V8-intern                                                       | 17         |
| `node:internal/modules/esm/utils` (Quelltext der geladenen ESM-Module) | 14         |
| `ManagedEndpointRuntime`, `sdk`, `server`, `runtimeLayer` Chunks       | je 2 bis 4 |

- Der Schema-Anteil ist von Upstream so gebaut (Contracts werden beim Import als Schema-AST erzeugt). Ihn zu senken heisst, Upstream-Code lazy zu machen; hohes Risiko und Merge-Aufwand, daher nicht angefasst.
- Worker-Threads: ca. 13 MB je Thread (gemessen), 6 Threads = ca. 78 MB. `resourceLimits` bringt nur ca. 2 MB je Thread. Die Reader starten schon lazy (`nodeSqliteClient.ts`, "idle slots never start a worker"); beim Start laufen so viele parallele Lesezugriffe, dass alle 4 anlaufen.
- Weitere Node-Flags (`--v8-pool-size=0`, `--max-lazy`, `--no-flush-bytecode`, `--no-opt --no-sparkplug`) senken den RSS nicht, teils +10 bis +25 MB.
- Whitespace-Minify des Bundles (esbuild, 20 auf 16 MB) senkte den RSS nicht, in der Messung sogar hoeher (396 statt 349 MB). Verworfen.
- Der Vergleichswert "Electron-Server 56 bis 61 MB" kann bei identischem Servercode nicht stimmen (allein Schema belegt 31 MB). Der Wert stammt vermutlich aus einer anderen Messmethode; der Electron-Server wurde nicht neu gemessen, weil das ein Schreiben in `~/.t3/userdata` verlangen wuerde (AGENTS.md verbietet es).

Fazit: Der Server liegt nach dem Flag bei ca. 340 MB RSS frisch gestartet; weitere Einsparungen erfordern Upstream-Eingriffe (Schema lazy, weniger Threads) oder ein V8-Startup-Snapshot. Der groessere Block ist der Webview (WebKitGTK, 110 bis 350 MB), siehe Phase 4.

## Nachtrag 2: Messung der neu gebauten App (Flag aktiv, kein Swap-Einfluss)

`perf-results/cc3-after-semispace.json`, 45 s nach Start plus 30 s Leerlauf:

| Gruppe                                                                     | PSS    |
| -------------------------------------------------------------------------- | ------ |
| Webview (WebKitWebProcess 333 + Network 30)                                | 363 MB |
| Server (`--max-semi-space-size=1` bestaetigt in der Prozess-Kommandozeile) | 320 MB |
| Node-Host                                                                  | 110 MB |
| Shell (cc3)                                                                | 95 MB  |
| Summe                                                                      | 888 MB |

- Die frueheren 8 MB fuer den Host waren ausgelagert (Swap); real sind es ca. 110 MB. Das Zusammenlegen von Host und Server (Phase 3.1) bleibt damit der zweitgroesste sichere Hebel.
- Shell mit 95 MB ist hoeher als erwartet (Tauri-Prozess selbst, GTK/WebKit-Bibliotheken).
- Groesster Block ist der Webview (Phase 4).

## Phase 3.1: Host (umgesetzt als Idle-Exit statt Prozess-Zusammenlegung)

Eine Integration in den Server-Prozess haette den Host-Code (Effect, Contracts, Schemas) in einem eigenen Bundle doppelt laden muessen (die Server-Chunks lassen sich nicht teilen) und haette nur die Node-Laufzeit gespart. Stattdessen beendet sich der Host selbst:

- `host/main.ts`: Exit nach 60 s ohne Anfrage, ausser (a) es laeuft eine Anfrage, (b) Snapshots sind aktiviert (`snapShotEnabled`, Watcher laufen im Host), (c) ein SSH-Tunnel wurde ueber `ensure-ssh*` geoeffnet.
- `host.rs`: Neustart bei der naechsten Anfrage; schlaegt das Schreiben fehl, weil der Host dazwischen beendet wurde, wird er einmal neu gestartet.
- `paths.rs`: Host startet mit `--max-semi-space-size=1` (119 auf 103 MB PSS, solange er laeuft).
- Selbsttest (`CC3_SELFTEST`): ein Aufruf nach 110 s prueft den Neustart (`late clientSettings null` im Log, Aufruf erfolgreich).

Messung (`perf-results/cc3-after-host-idle.json`, nach 130 s Leerlauf): Summe 807 MB (vorher 888 MB): Webview 363, Server 348, Shell 97, Host 0.
Mit aktivierten Snapshots oder SSH bleibt der Host dauerhaft (ca. 100 MB).
