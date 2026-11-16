# Server-Speicher (Phase 2.1)

Messung: frischer Server (gebuendelter Node aus der installierten CC3-App, temporaeres Home, 26 bis 50 s Leerlauf), `process.memoryUsage()` pro Thread.
Skripte: `apps/cc3-desktop/scripts/perf/server-standalone.mjs` (startet den Server wie `server.rs`), `server-mem-preload.cjs` (loggt Speicher je Thread), `measure.mjs` (PSS des Prozessbaums).

## Befunde

- Hauptthread: V8-Heap reserviert 230 MB bei nur 118 MB belegt (grosse Young Generation), external 41 MB.
- Sechs weitere Worker-Threads (je ca. 7 MB Heap, aber je eigene Isolate): SQLite-Reader/Writer-Worker (`packages/shared/src/nodeSqliteClient.ts`, aktiviert in `apps/server/src/persistence/Sqlite.ts`) plus Checkpoint-Worker.
- Das laufende CC3 hatte ca. 130 MB des Server-Prozesses ausgelagert (Swap), Rechner unter Speicherdruck (13 GB belegt, 14 GB Swap). Messungen am laufenden System sind deshalb unzuverlaessig; frische Standalone-Laeufe sind die Referenz.
- Im Prozessbaum der laufenden App hingen ausserdem Agent-Kindprozesse (`claude`, `node`, `bun`, MCP), die kein Server-Overhead sind und getrennt gezaehlt werden muessen (`measure.mjs` gruppiert jetzt in `byGroup`).
- Node-Host (`host/main.mjs`) belegt nur ca. 8 MB PSS im Leerlauf, nicht 100 MB (frueherer Wert war RSS). Der Gewinn aus dem Zusammenlegen (Phase 3.1) ist damit kleiner als geplant.

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
