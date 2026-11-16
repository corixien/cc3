# CC3 Performance Plan (Branch `cc3-performance`)

Status: reiner Plan. In diesem Branch wird noch kein Programmcode geaendert. Dieses Dokument zeigt auf bestehende Dateien und beschreibt, was dort spaeter zu tun ist. Es schreibt nichts um.

Ziel: CC3 so schnell und speichersparend wie moeglich machen, bei **pixelgleichem Aussehen** (jede Farbe, jeder Effekt, jede Animation, jedes Pop-up) und **unveraenderter Funktion**. Aufwand ist zweitrangig.

---

## 0. Ausgangslage (gemessen und gezaehlt)

### 0.1 Aufbau von CC3 heute

| Teil               | Ort                                                                                                                                 | Rolle                                                                                                              |
| ------------------ | ----------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------ |
| Tauri-Shell (Rust) | `apps/cc3-desktop/src-tauri/src/` (`main.rs`, `native.rs`, `host.rs`, `server.rs`, `links.rs`, `alert.rs`, `paths.rs`, `bridge.js`) | Fenster, Tray-lose Shell, native Operationen, Alert-Fenster, Deep Links, Badge                                     |
| Node-Host-Sidecar  | `apps/cc3-desktop/host/main.ts`, `host/electron-stub.mjs`, `vite.config.ts`                                                         | fuehrt die Electron-IPC-Logik von `apps/desktop` mit gestubbtem Electron aus (CLI-Installer, Snapshots, Web-Links) |
| Server             | `apps/server/src/` (ca. 293 000 Zeilen TS)                                                                                          | Orchestrierung, Provider, Git, Terminal, Preview-Browser                                                           |
| Web-UI             | `apps/web/src/` (668 TSX-Komponenten, ca. 317 000 Zeilen TS/TSX, 2 700 Zeilen CSS)                                                  | die gesamte Oberflaeche                                                                                            |
| Packaging          | `apps/cc3-desktop/scripts/stage.mjs`                                                                                                | `vp pack`, Stage, `tauri build --bundles deb`                                                                      |

### 0.2 Gemessener RAM (Leerlauf, PSS-Summe)

|                      | CC3 (Tauri)    | T3 (Electron)      |
| -------------------- | -------------- | ------------------ |
| Gesamt               | 650 bis 850 MB | ca. 350 MB         |
| Shell/Main           | 72 bis 92 MB   | 63 MB              |
| Renderer / WebKitGTK | 110 bis 350 MB | 189 MB + 23 MB GPU |
| Node-Host            | ca. 100 MB     | entfaellt          |
| Server               | 320 bis 400 MB | 56 bis 61 MB       |

Die Ursache der Server-Differenz ist **ungeklaert** (`MALLOC_ARENA_MAX` und `--max-old-space-size` brachten nichts). Das ist der groesste einzelne Hebel.

### 0.3 Bekannte Schwachstellen

- WebKitGTK rendert langsamer als Chromium und war Ursache fuer das unscharfe Bild (siehe Preview-Abschnitt in `project-resume.md`).
- Zwei Node-Prozesse (Host und Server) statt einem.
- Schwere Abhaengigkeiten in `apps/web/package.json`, die vermutlich frueh geladen werden: `pdfjs-dist`, `@betteroffice/docx|pptx|xlsx`, `@clerk/*`, `katex`, `sql.js`, `heic-to`, `tiff`, `jszip`, `canvas-confetti`, `@tiptap/*`, `@pierre/diffs`, `@pierre/trees`. `apps/web/dist` ist 255 MB gross.
- Sehr grosse Einzeldateien: `components/ChatView.tsx` (4 085 Zeilen), `chat/ChatComposer.tsx` (3 846), `Sidebar.tsx` (3 662), `CommandPalette.tsx` (3 080), `settings/SettingsPanels.tsx` (2 376), `ComposerPromptEditorTiptap.tsx` (2 009), `ThreadTerminalDrawer.tsx` (1 956), `ChatMarkdown.tsx` (1 924), `chat/MessagesTimeline.tsx` (1 891).
- Server: `orchestration-v2/Orchestrator.ts` (16 432 Zeilen), `Adapters/ClaudeAdapterV2.ts` (14 469), `ProjectionStore.ts` (9 346), `terminal/Manager.ts` (3 871), `ws.ts` (3 713).

---

## 1. Grundprinzipien

1. **Erst messen, dann aendern.** Keine Optimierung ohne Messung davor und danach.
2. **Pixelgleichheit ist ein harter Test, kein Wunsch.** Jede Phase muss die Golden-Screenshot-Suite (Phase 1) bestehen, sonst wird sie zurueckgedreht.
3. **Kleine, einzeln umkehrbare Schritte.** Ein Commit pro Massnahme, jeweils mit Messwerten in der Commit-Beschreibung.
4. **Reihenfolge nach Nutzen pro Risiko.** Server und Shell zuerst (grosser Gewinn, null Einfluss auf das Aussehen). Web-UI danach (gleiches DOM, gleiches Aussehen). Native UI nur, wenn die Budgets sonst nicht erreicht werden (Entscheidungstor in Phase 7).
5. **Merge-Faehigkeit mit dem Original erhalten.** Aenderungen an Dateien aus dem Upstream moeglichst klein halten.
6. **AGENTS.md beachten:** keine Prozesse per Muster killen, nie in `~/.t3/userdata` schreiben, vor dem Push den Web-Typecheck.

---

## 2. Leistungsbudgets (Zielwerte)

Die Zahlen sind Ziele, die in Phase 0 gegen die gemessene Basis angepasst werden.

| Metrik                                  | Basis heute                    | Ziel nach Phase 5                     | Ziel nach Phase 7 (nur bei nativer UI) |
| --------------------------------------- | ------------------------------ | ------------------------------------- | -------------------------------------- |
| RAM gesamt, Leerlauf                    | 650 bis 850 MB                 | unter 350 MB                          | unter 250 MB                           |
| RAM Server                              | 320 bis 400 MB                 | unter 120 MB                          | unter 120 MB                           |
| Prozesse                                | Shell, Host, Server, WebKit x3 | Shell, Server, Webview                | Shell, Server                          |
| Kaltstart bis erster Frame              | zu messen                      | unter 1,0 s                           | unter 0,4 s                            |
| Kaltstart bis bedienbar (Chat sichtbar) | zu messen                      | unter 2,0 s                           | unter 0,8 s                            |
| Thread mit 5 000 Nachrichten oeffnen    | zu messen                      | unter 400 ms                          | unter 150 ms                           |
| Scroll in langem Thread                 | zu messen                      | 60 fps ohne Drops                     | 120 fps                                |
| Tippen im Composer, Eingabe bis Pixel   | zu messen                      | unter 16 ms                           | unter 8 ms                             |
| JS-Bundle beim Start                    | zu messen                      | unter 40 Prozent der heutigen Groesse | entfaellt                              |
| CPU im Leerlauf                         | zu messen                      | unter 1 Prozent                       | unter 0,5 Prozent                      |

---

## 3. Phase 0: Messwerkzeug und Basis

Ohne diese Phase ist alles Weitere Raten.

### 0.1 Reproduzierbares Messskript

- Neues Skript `apps/cc3-desktop/scripts/perf/measure.mjs` (neu, nicht in Upstream-Dateien).
- Misst nach festem Ablauf: Start, 60 s Leerlauf, Thread oeffnen, Scrollen, Tippen, Preview oeffnen, Terminal oeffnen.
- RAM pro Prozess aus `/proc/<pid>/smaps_rollup` (PSS), Summe ueber alle Kind-Prozesse des Shell-PIDs. Immer nur exakte PIDs, nie Mustererkennung.
- CPU-Zeit aus `/proc/<pid>/stat`, Startzeit aus Zeitstempel im Log (`bridge.js` schreibt bereits ueber `log`).
- Ausgabe als JSON nach `perf-results/<datum>-<commit>.json`, plus Markdown-Vergleichstabelle.
- Testdaten-Fixtures: ein synthetischer Thread mit 5 000 Nachrichten (Text, Code, Diffs, Tool-Aufrufe, Bilder), ein Projekt mit 10 000 Dateien. Erzeugt per Skript in einem **temporaeren** Home (`T3CODE_HOME` setzen, nie das echte Home verwenden).

### 0.2 Vergleichsmessung gegen Electron

- Gleiches Skript gegen die originale T3-App (nur lesen, nichts schreiben).
- Ergebnis: Tabelle in `docs/perf/BASELINE.md`.

### 0.3 Profilwerkzeuge festlegen

- Server: `node --cpu-prof --heap-prof`, `process.memoryUsage()` als periodischer Log (`apps/server/src/resourceTelemetry` existiert bereits, zuerst anschauen).
- Web: Chrome DevTools Performance (im Chromium-Spike), `performance.mark/measure` in Startpfaden, React Profiler.
- Rust-Shell: `perf`, `heaptrack` falls vorhanden.

**Abnahme Phase 0:** Basiswerte liegen als Datei vor, Skript laeuft dreimal hintereinander mit weniger als 5 Prozent Streuung.

---

## 4. Phase 1: Pixelgleichheits-Sicherung (Golden Suite)

Dies ist die Grundlage fuer die Zusage "jede Farbe, jeder Effekt, jede Animation, jedes Pop-up gleich".

### 1.1 Zustandskatalog

Alle Bildschirme und Zustaende werden als Liste in `docs/perf/STATE-CATALOG.md` erfasst. Quelle ist der Code:

- Routen: `apps/web/src/routes/` (`_chat.$environmentId.$threadId.tsx`, `_chat.draft.$draftId.tsx`, `_chat.index.tsx`, `_chat.new.tsx`, `_chat.pull-requests.tsx`, alle `_chat.settings.*.tsx`, `approve-agent.tsx` und weitere).
- Komponentenordner in `apps/web/src/components/`: `auth`, `chat`, `cloud`, `desktop`, `device`, `diffs`, `files`, `guest`, `mcp`, `media`, `onboarding`, `permissions`, `preview`, `pullRequest`, `search`, `settings`, `sidebar`, `terminal`, `ui`, `usage`.
- Zustaende je Komponente: leer, laden, Fehler, Hover, Fokus, aktiv, deaktiviert, gedrueckt, Auswahl, Drag.
- Alle Themes, hell und dunkel, alle Schrift- und Kontrast-Einstellungen (`appearanceFonts.ts`, `appearanceContrast.ts`), alle Zoomstufen, Fenster maximiert und normal, frameless Ecken.
- Alle Dialoge, Popups, Toasts, Kontextmenues, Tooltips, Banner (`chat/ComposerBanner.tsx`, `ChatBannerLane`), Alert-Karte (`alert.rs`).

### 1.2 Golden Screenshots

- Mit Playwright (Chromium, fester Viewport, feste Schrift, deaktivierte Systemanimationen, fester Zeitgeber) wird jeder Zustand aus 1.1 aufgenommen. Fixtures liefern deterministische Daten.
- Basis-Referenz = Aussehen **vor** den Aenderungen (Commit `599ee5cc` bzw. der Stand beim Branch-Start).
- Ablage: `apps/web/golden/` (per Git LFS oder als externes Artefakt, nicht im normalen Git, wenn zu gross).
- Vergleich per Pixeldiff mit fester Toleranz (Ziel: 0 abweichende Pixel; wo Kantenglaettung variiert, maximal 0,1 Prozent und nie Farbabweichung).

### 1.3 Animationsvergleich

- Alle Animationen werden aus dem Code extrahiert: `@keyframes` in `apps/web/src/index.css` (16 Stueck) und weitere in Komponenten, `transition-*` und `animate-*` in Tailwind-Klassen, JS-Animationen (`AnimatedHeight.tsx`, `AnimatedWidth.tsx`, `Sidebar.motion.ts`, `canvas-confetti`, `HistorySwipeNavigation.tsx`).
- Fuer jede: Dauer, Easing, Start- und Endzustand in `docs/perf/ANIMATION-CATALOG.md`.
- Test: pro Animation Bildfolge bei festen Zeitpunkten (0, 25, 50, 75, 100 Prozent) via Web Animations API oder per Zeitsteuerung aufnehmen und gegen die Referenz vergleichen.

### 1.4 Funktionstests

- Vorhandene Tests sind die Basis: `*.test.ts`, `*.browser.test.tsx` im Web (z. B. `Sidebar.*.browser.test.tsx`, `ChatView.*.browser.test.tsx`, `ComposerPromptEditorTiptap.*.browser.test.tsx`, `ThreadTerminalDrawer.browser.test.tsx`, `CommandPalette.*.browser.test.tsx`), Tests im Server.
- Bekannte Luecke: `ComposerBannerStack.browser.test.tsx` ist nach der Banner-Aenderung moeglicherweise veraltet. Aktualisieren, bevor irgendeine Phase startet.
- Zusaetzlich ein End-zu-End-Ablauf pro Hauptfunktion (Checkliste in Abschnitt 12).

**Abnahme Phase 1:** Die Suite laeuft gruen gegen den unveraenderten Stand und ist in `vp run`-Skripten erreichbar (`perf:golden`, `perf:anim`).

---

## 5. Phase 2: Server abspecken (null Auswirkung auf das Aussehen)

Groesster messbarer Gewinn, deshalb zuerst.

### 2.1 Speicher aufklaeren

Ziel: herausfinden, warum der Server in CC3 320 bis 400 MB belegt und in Electron etwa 60 MB.

1. Zuerst die Unterschiede festhalten: Node-Version und Startflags, Umgebungsvariablen, Arbeitsverzeichnis, Anzahl Kind-Prozesse, Datenbankgroesse. In Electron laeuft der Server als Kindprozess des Hauptprozesses; in CC3 als Kind von `server.rs`. Pfad: `apps/cc3-desktop/src-tauri/src/server.rs`, `paths.rs`.
2. Im Server periodisch `process.memoryUsage()` (rss, heapUsed, external, arrayBuffers) und `process.report` bzw. `--heap-prof` ausgeben. Einstieg: `apps/server/src/resourceTelemetry`, `apps/server/src/diagnostics`.
3. Trennen: V8-Heap, ArrayBuffers/Buffer, native Module, Thread-Stacks, gemappte Dateien. Mit `/proc/<pid>/smaps` die groessten Regionen pro Mapping ausweisen.
4. Der Inspector-Anhang per `SIGUSR1` hat im frueheren Versuch gehaengt. Stattdessen den Server mit `--inspect=0` oder `--heap-prof` **sauber starten**, nicht nachtraeglich anhaengen.
5. Ergebnis: eine Tabelle "Verbraucher, MB, Ursache" in `docs/perf/SERVER-MEMORY.md`.

Verdaechtige, die geprueft werden muessen (Reihenfolge nach Wahrscheinlichkeit):

- Native Module und Bibliotheken, die beim Start geladen werden: `playwright-core`, `node-pty`, `@napi-rs/keyring`, `ffi-rs`, `transcribe-cpp`, `@ff-labs/fff-node`, `undici`.
- Eingebettete Agent-SDKs: `@anthropic-ai/claude-agent-sdk`, `@cursor/sdk`, `@muse-code/sdk`. Diese ziehen grosse Abhaengigkeitsbaeume nach.
- Provider-Pakete (`packages/provider-*`, `packages/source-control-*`), die alle beim Start instanziiert werden, auch wenn nur einer genutzt wird.
- `orchestration-v2` (`Orchestrator.ts`, `ProjectionStore.ts`): komplette Projektionen im Speicher statt bedarfsweise.
- SQLite-Zustand und Caches (`apps/server/src/persistence`), Datei-Beobachter (`launcherWatch.ts`, `workspace`), Prozess-Telemetrie.
- Der Preview-Stream (`apps/server/src/preview/ServerBrowserStream.ts`), JPEG-Puffer, Qualitaet wurde auf 92 angehoben (hoeherer Speicher/CPU), dann pruefen, ob adaptive Qualitaet noetig ist, ohne die Schaerfe zu veraendern.

### 2.2 Massnahmen (nach Befund auszuwaehlen)

| Massnahme                                                                                                               | Wo                                                                                  | Risiko                                      |
| ----------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------- | ------------------------------------------- |
| Provider und SDKs erst beim ersten Gebrauch laden (dynamisches `import()`), nicht beim Start                            | `apps/server/src/provider/`, Registrierung in `server.ts`/`serverRuntimeStartup.ts` | niedrig, wenn Fehlerpfade identisch bleiben |
| Playwright nur starten, wenn eine Vorschau geoeffnet wird                                                               | `apps/server/src/preview/`                                                          | niedrig                                     |
| Zustandsprojektionen paginieren oder bedarfsweise laden                                                                 | `orchestration-v2/ProjectionStore.ts`, `Orchestrator.ts`                            | mittel, gruendlich testen                   |
| Chat-Historien nicht komplett im Speicher halten (Streaming aus SQLite)                                                 | `persistence`, `ws.ts`                                                              | mittel                                      |
| Node-Flags: `--max-semi-space-size`, `--single-threaded-gc`, `--v8-pool-size`, Compile-Cache bleibt (`compileCache.ts`) | Start in `server.rs`                                                                | niedrig                                     |
| `libuv`-Threadpool (`UV_THREADPOOL_SIZE`) begrenzen                                                                     | `server.rs`                                                                         | niedrig                                     |
| Bundle mit Tree-Shaking und ohne Dev-Abhaengigkeiten ausliefern (`pnpm deploy --prod` bereinigen)                       | `scripts/stage.mjs`                                                                 | niedrig                                     |
| `external`-Speicher reduzieren (Buffer-Pools, Streams schliessen)                                                       | gefundene Stellen                                                                   | niedrig bis mittel                          |

### 2.3 Server-CPU

- Hot-Path-Profil bei Streaming-Antworten und grossen Diffs (`vcs/GitVcsDriverCore.ts`, `git/GitManager.ts`): `--cpu-prof`.
- Batchen und Drosseln der WebSocket-Nachrichten (`ws.ts`) pruefen: weniger Frames bei gleicher sichtbarer Aktualitaet. Die Aktualisierungsrate im UI darf sich nicht sichtbar aendern.
- Serialisierung (JSON) bei grossen Payloads pruefen.

**Abnahme Phase 2:** Server-RAM im Leerlauf unter 120 MB, alle Server-Tests gruen, Golden Suite unveraendert.

---

## 6. Phase 3: Prozess-Struktur (Shell und Host)

### 3.1 Node-Host entfernen oder verschlanken

Heute: ein eigener Node-Prozess (`host/main.ts`, ca. 100 MB) fuehrt die Desktop-IPC-Logik aus.

Optionen, nach Aufwand und Gewinn:

1. **Host in den Server-Prozess einhaengen** (empfohlen): Die benoetigten Dienste (CLI-Installer, Snapshots, Web-Links) werden vom Server-Prozess selbst bereitgestellt, z. B. ueber einen zusaetzlichen RPC-Kanal. Spart eine komplette Node-Laufzeit. Eingriffspunkte: `host/main.ts`, `host.rs`, `server.rs`, Ladeort in `apps/server/src/serverRuntimeStartup.ts`.
2. Host nur bei Bedarf starten (lazy): CLI-Installer und Snapshot werden selten gebraucht. `host.rs` startet den Prozess erst beim ersten `cc3-host`-Aufruf und beendet ihn nach Leerlauf.
3. Teile der Host-Logik nach Rust portieren (CLI-Installer ist einfach, Snapshots nicht): nur wenn 1 und 2 nicht reichen.

### 3.2 Shell-Feinschliff (Rust)

- Release-Profil in `src-tauri/Cargo.toml`: `lto = "fat"`, `codegen-units = 1`, `opt-level = 3` (oder `"s"` fuer Groesse), `panic = "abort"`, `strip = true`.
- Tauri-Plugins pruefen und ungenutzte entfernen (`devtools` nur in Dev-Builds).
- `bridge.js` (20,8 KB) klein halten: kein Polling, Ereignisse statt Timer, Selbsttest nur in Debug.
- Alert-Fenster (`alert.rs`) erst beim ersten Alert erzeugen und danach verstecken statt neu erzeugen (spart Startkosten bei jedem Alert); Speicher des versteckten Fensters messen und gegen Neuerzeugung abwaegen.
- `kdotool`-Aufrufe nicht blockierend.

### 3.3 Orphan-Server

- Bekannte Luecke: bei SIGTERM bleibt der Server als Waise zurueck (nur `RunEvent::Exit` raeumt auf). Signal-Handler in `main.rs` ergaenzen und einen PID-Wachhund einbauen. Wichtig, damit ein Neustart nie zwei Server (doppelter RAM) hinterlaesst.

**Abnahme Phase 3:** Ein Node-Prozess weniger oder ein lazy gestarteter, Gesamt-RAM um mindestens 100 MB tiefer, alle Host-Funktionen (CLI-Installer, Snapshots, Deep Links, Badge) weiterhin ok.

---

## 7. Phase 4: Webview-Entscheidung (Rendering-Engine)

Das ist die wichtigste Architekturentscheidung fuer die UI-Leistung bei **identischem** Aussehen.

### 4.1 Problem

WebKitGTK (System-WebView) ist unter Linux oft langsamer als Chromium, hat schlechtere GPU-Beschleunigung und weicht beim Rendering leicht ab (Kantenglaettung, Filter, `backdrop-filter`, Schriftmetriken). Das gefaehrdet die Pixelgleichheit gegenueber dem Original (Chromium/Electron).

### 4.2 Kandidaten und Spike

Jeweils ein kleiner Spike (1 bis 2 Tage), gemessen mit Phase-0-Skript und Golden Suite:

| Kandidat                                                 | Idee                                 | Erwartung                                                                        |
| -------------------------------------------------------- | ------------------------------------ | -------------------------------------------------------------------------------- |
| A. Tauri + WebKitGTK (heute)                             | Referenz                             | RAM niedrig bis mittel, Leistung schwach                                         |
| B. Tauri mit CEF-Runtime (Chromium Embedded Framework)   | gleiche Tauri-Shell, anderer Webview | Rendering wie Electron (Pixelgleichheit am wahrscheinlichsten), RAM wie Electron |
| C. Electron zurueck, aber mit schlankem Server (Phase 2) | Referenz der Original-App            | RAM-Vergleichswert                                                               |
| D. System-Chromium/Chrome im App-Modus als Fenster       | `--app=` ohne Electron               | RAM gering, Integrationsaufwand hoch                                             |
| E. Servo                                                 | wenn die Linux-Reife es zulaesst     | experimentell, nur beobachten                                                    |

Pruefpunkte je Kandidat: Golden-Suite-Treffer, Scroll-fps, Eingabe-Latenz, RAM, Startzeit, Webview-Anbindung der `window.desktopBridge` (`bridge.js`), transparentes Fenster mit abgerundeten Ecken (`clip-path` plus `::after` Rand), `windowControlsOverlay`-Polyfill, Drag-Region, Alert-Fenster, WebRTC/Audio fuer den Alert-Ton, Schriftrendering von Lexend Deca.

### 4.3 Entscheidungsregel

- Wenn B die Golden-Suite bestanden und mindestens 30 Prozent bessere Rendering-Werte als A liefert: B uebernehmen.
- Wenn A trotz Optimierung (Phase 5) die Budgets erreicht: A behalten.
- Ergebnis und Messwerte in `docs/perf/WEBVIEW-DECISION.md`.

### 4.4 Chromium-Feintuning (falls B oder C)

- Startflags: `--disable-features=...` (nur Funktionen, die CC3 nicht braucht), `--js-flags=--max-lazy`, Renderer-Prozesslimit, Hardwarebeschleunigung pruefen, `--enable-gpu-rasterization`.
- Nur Flags, die weder Aussehen noch Funktion aendern.

---

## 8. Phase 5: Web-UI beschleunigen (gleiches DOM, gleiches Aussehen)

Alle Massnahmen in dieser Phase aendern **nicht** das gerenderte Ergebnis. Die Golden Suite entscheidet.

### 5.1 Start und Bundle

1. **Bundle analysieren**: `vite build` mit `rollup-plugin-visualizer` fuer `apps/web`. Ergebnis `docs/perf/BUNDLE.md`.
2. **Code-Splitting nach Route und Funktion** (`apps/web/src/routes/*`): jede Route und jede Settings-Seite laedt ihren Code erst beim Oeffnen. `@tanstack/react-router` unterstuetzt lazy Routes.
3. **Schwere Bibliotheken lazy laden**, erst beim ersten Gebrauch:
   - Dokumentvorschau: `pdfjs-dist`, `@betteroffice/docx`, `@betteroffice/pptx`, `@betteroffice/xlsx`, `jszip`, `sql.js`, `heic-to`, `tiff` (Ort: `components/files/FilePreviewPanel.tsx` und `components/media`).
   - `@clerk/*` nur wenn Cloud-Login aktiv ist (Ort: `components/clerk`, `components/cloud`).
   - `katex` nur wenn eine Nachricht Formeln enthaelt (`ChatMarkdown.tsx`).
   - `canvas-confetti` nur beim Ausloesen.
   - `@pierre/diffs` und `@pierre/trees` nur beim Oeffnen des Diff-/Dateipanels (`DiffPanel.tsx`, `components/diffs`).
   - `ghostty`-Terminal (`apps/web/src/ghostty`, `components/terminal`) nur beim Oeffnen eines Terminals.
   - `@tiptap/*` im Composer: bleibt im Startpfad (Composer ist sofort sichtbar), aber nur benoetigte Erweiterungen laden.
   - `@betteroffice/fonts-cjk`: nur bei CJK-Text.
4. **Schriften**: Lexend Deca als variable Schrift (`apps/web/public/fonts/LexendDeca.ttf`) in `woff2` konvertieren und per `<link rel=preload>` laden; `font-display: swap` nur wenn kein sichtbarer Sprung entsteht (sonst `block` mit Preload). `@fontsource-variable/inter` pruefen, ob noch gebraucht.
5. **Startpfad**: `appInit.tsx`, `bootstrap.ts`, `AppRoot.tsx`: Arbeit parallelisieren, nichts Unnoetiges vor dem ersten Frame (nur Layout-Gerueste und Theme). Kritisches CSS inline.
6. **V8 Code-Cache**: bei CEF/Chromium aktivieren; bei WebKit entfaellt.
7. **Statische Ressourcen** ueber `tauri://`-Protokoll bzw. lokalen Server mit Kompression (br/gzip) und `Cache-Control`-Immutable ausliefern.

### 5.2 Rendering und React

1. **Profilieren**: React Profiler auf Thread oeffnen, Scrollen, Tippen, Streaming-Antwort, Sidebar mit vielen Threads.
2. **Listen virtualisieren**: `@legendapp/list` ist schon da. Pruefen, wo es noch nicht genutzt wird: `chat/MessagesTimeline.tsx` (1 891 Zeilen), `Sidebar.tsx` (Threads, Projekte), `CommandPalette*.tsx`, Dateibaum, PR-Listen, Diffs.
3. **Gezielte Memoisierung und Selektoren**: grosse Komponenten (`ChatView.tsx`, `chat/ChatComposer.tsx`, `Sidebar.tsx`) auf unnoetige Re-Renders pruefen (zustand-Selektoren, `useSyncExternalStore`, `React.memo`, stabile Referenzen). Kein Umbau der Struktur, nur gezielte Eingriffe an den heissen Stellen.
4. **Streaming-Updates drosseln**: Token-Updates von Antworten in `requestAnimationFrame`-Batches buendeln (`@tanstack/react-pacer` ist vorhanden). Die sichtbare Wortfolge darf sich nicht aendern.
5. **Markdown-Pipeline** (`ChatMarkdown.tsx`: `react-markdown`, `remark-*`, `rehype`): Ergebnis cachen pro Nachricht, fertige Nachrichten nie neu parsen, nur die letzte streamende Nachricht inkrementell. Syntax-Hervorhebung (`composerCodeBlockHighlight.ts`, `diffs`) in Web Worker (`DiffWorkerPoolProvider.tsx` zeigt schon ein Worker-Muster).
6. **Layout-Thrash vermeiden**: Messungen (`getBoundingClientRect`, `ResizeObserver`) bei `AnimatedHeight.tsx`, `AnimatedWidth.tsx`, `Sidebar.motion.ts`, `RightPanelSheet.tsx` buendeln.
7. **CSS**: `will-change` und `contain: layout paint` an stabilen Containern (Sidebar, Timeline, Panels), `content-visibility: auto` fuer lange Listen, **nur wenn** die Golden Suite keine Abweichung zeigt. Teure Effekte (`backdrop-filter`, grosse Schatten, Blur) auf Einzelkosten messen und bei gleichem Ergebnis auf GPU-Ebenen legen. `html { clip-path }` (Fensterecken) auf Kosten pruefen und ggf. durch `border-radius` auf einem Container ersetzen, falls Pixelgleichheit bleibt.
8. **Bilder und Medien**: Vorschaubilder in passender Groesse, `decoding="async"`, `loading="lazy"`, HEIC/TIFF-Dekodierung in Worker.
9. **Terminal** (`ThreadTerminalDrawer.tsx`, `ghostty`): Schreibpuffer pro Frame buendeln, unsichtbare Terminals pausieren (`PersistentThreadTerminals.tsx`), Scrollback begrenzen.
10. **Speicher**: Event-Listener und Subscriptions beim Unmount pruefen (Leckjagd mit Heap-Snapshots nach 20 Thread-Wechseln), grosse Stores (`composerDraftStore.ts`, `diffPanelStore.ts`, Browser-Stores) auf Wachstum pruefen.

### 5.3 Preview-Browser

- `apps/server/src/preview/ServerBrowserStream.ts` und `apps/web/src/browser/ServerBrowserSurface.tsx`: Frame-Rate adaptiv (nur bei Aenderung senden), Differenzbilder, optional WebP/H.264 statt JPEG, **ohne** die Schaerfe zu verlieren (aktuell Qualitaet 92, DPR-Cap 2). Vergleich per Golden-Screenshots der Vorschau.

### 5.4 Usage, Notifications und Alert

- Poll-Intervalle in `components/usage/*` und `threadNotifications.ts` pruefen (Server-Push statt Polling wo moeglich).
- Alert-Ton: `AudioContext` einmal erzeugen und wiederverwenden.

**Abnahme Phase 5:** Budgets aus Abschnitt 2 (Spalte "nach Phase 5") erreicht oder dokumentiert verfehlt, Golden Suite und Animations-Suite unveraendert gruen, alle bestehenden Browser-Tests gruen.

---

## 9. Phase 6: Packaging und Startzeit

1. **Binaergroesse und Kaltstart**: Server als einzelne gebuendelte Datei mit V8-Snapshot oder Compile-Cache (`cliCompileCache.ts`, `compileCache.ts` existieren) vorbereiten, Cache beim Build erzeugen statt beim ersten Start.
2. **Parallelstart**: Fenster und Server gleichzeitig starten, Fenster zeigt das Geruest sofort (kein weisser Blitz, dunkler Hintergrund gleich dem Theme, Transparenz-Flackern pruefen).
3. **Abhaengigkeiten bereinigen**: `scripts/stage.mjs` mit `pnpm deploy --prod`, `node_modules` ohne Tests, Doku, Quellkarten, nicht benutzte Plattform-Binaries (nur `linux-x64`). Ziel: deutlich kleineres `.deb` (heute 245,65 MiB).
4. **Warmstart**: Zustand der UI speichern und beim naechsten Start sofort herstellen (Thread-Liste aus lokalem Cache zeigen, waehrend der Server verbindet), nur wenn die Anzeige identisch bleibt.
5. **Weitere Formate** (offene Luecken): AppImage, arm64, Auto-Update. Gehoeren zur Auslieferung, nicht zur Leistung, aber der Auto-Update-Weg muss die schlanke Struktur beibehalten.

---

## 10. Phase 7: Entscheidungstor "Native Oberflaeche"

Nur wenn nach Phase 5 und 6 die Budgets (insbesondere RAM unter 250 MB, Frame-Zeiten) nicht erreicht sind **und** der Nutzer den Mehraufwand ausdruecklich freigibt.

### 7.1 Was eine native UI bedeutet

Die Web-UI wird durch eine GPU-beschleunigte Rust-Oberflaeche ersetzt. Der Server bleibt unveraendert (gleiche WebSocket-/RPC-Schnittstelle, Vertraege aus `packages/contracts`, Client-Logik aus `packages/client-runtime`).

### 7.2 Voraussetzungen fuer Pixelgleichheit

1. **Design-Token-Extraktion**: alle Farben, Abstaende, Radien, Schatten, Schriftgroessen aus `apps/web/src/index.css` (Tailwind-Variablen, `--font-sans`, `--font-mono`, Theme-Paletten) automatisch in eine Rust-Tokentabelle exportieren (Skript), nie von Hand.
2. **Layout-Motor mit CSS-Flexbox-/Grid-Semantik**: `taffy` (wie bei Dioxus/Blitz/Bevy) fuer identisches Layout.
3. **Text**: `cosmic-text` oder `parley` mit denselben Schriften (Lexend Deca, Liberation Mono), gleiche Zeilenhoehen und Umbruchregeln. Kantenglaettung und Subpixel-Positionierung gegen Chromium abgleichen.
4. **Zeichnen**: `vello` (GPU, wgpu) oder Skia, fuer Ebenen, Verlaeufe, Blur, Schatten, Clip-Pfade, Squircle-Ecken.
5. **Animationen**: gleiche Kurven (`cubic-bezier`-Werte aus dem Animationskatalog), gleiche Dauer, gleiche Start-/Endzustaende, Zeitsteuerung ueber einen Frame-Scheduler.
6. **Komponenten**, aufgeteilt nach Aufwand:
   - Einfach: Buttons, Schalter, Slider, Dialoge, Toasts, Tooltips, Badges, Karten, Banner, Menues.
   - Mittel: Sidebar mit Drag-and-drop, Command Palette, Einstellungen (ca. 20 Seiten), Onboarding, PR-Ansichten, Dateibaum.
   - Schwer: Chat-Timeline mit Markdown (Tabellen, Code, Formeln, Bilder), Diff-Ansicht, Composer mit Rich-Text-Editor (Tiptap-Ersatz: Mentions, Chips, Listen, Code-Bloecke, Undo-Gruppen, IME, Text-Richtung), Terminal (libghostty-Anbindung), Dokumentvorschau (PDF/DOCX/XLSX/PPTX), Preview-Browser (weiterhin Frame-Stream oder WebView-Insel).
7. **Barrierefreiheit, IME, Drag-and-drop, Auswahl, Kontextmenues, Tastaturkuerzel**: Pflicht, nicht optional (`accesskit`, `keybindings.ts`).

### 7.3 Vorgehen

1. Sechs-Wochen-Spike: Sidebar + leerer Chat + Composer in nativ, gegen Golden Suite messen. Danach neu entscheiden.
2. Parallelbetrieb: Web-UI bleibt als Rueckfallebene (`--ui=web|native`), pro Bildschirm schrittweise Umschaltung.
3. Jede Komponente braucht Golden-Test und Funktionstest, bevor sie die Web-Version ersetzt.
4. Upstream-Aenderungen: Vergleichsskript, das neue/geaenderte Web-Komponenten markiert und in eine Nacharbeitsliste schreibt.

**Aufwand (grobe Schaetzung):** mehrere Monate. Ohne ausdrueckliche Freigabe wird diese Phase nicht begonnen.

---

## 11. Risiken

| Risiko                                                | Gegenmassnahme                                                                                      |
| ----------------------------------------------------- | --------------------------------------------------------------------------------------------------- |
| Optimierung aendert das Aussehen unbemerkt            | Golden Suite pro Commit, Toleranz 0 bei Farben                                                      |
| Lazy Loading verursacht sichtbares Nachladen (Pop-in) | Preload bei Hover/Idle, Skelettzustand identisch zum jetzigen Ladezustand, Messung der Verzoegerung |
| Server-Umbau (Lazy Provider) bricht seltene Pfade     | alle Server-Tests, Smoke-Test je Provider (soweit installiert)                                      |
| Webview-Wechsel (CEF) verschlechtert Integration      | Spike vor Entscheidung, Rueckfall auf WebKitGTK                                                     |
| Merge-Konflikte mit Upstream                          | kleine Eingriffe, keine Massen-Umformatierung, Rebase-Skript                                        |
| Messwerte schwanken                                   | feste Hardware, Mittel aus 5 Laeufen, Leerlauf nach 60 s                                            |
| Tests laufen gegen echte Nutzerdaten                  | immer temporaeres `T3CODE_HOME`, nie `~/.t3/userdata`, nie die Neon-Produktionsdatenbank            |
| Prozesse falsch beendet                               | nur exakte PIDs, nie Muster-Kills (AGENTS.md)                                                       |

---

## 12. Funktions-Checkliste (muss nach jeder Phase vollstaendig bestehen)

- Chat: Nachricht senden, Streaming, Abbrechen, Tool-Aufrufe, Fragen, Plan-Modus, Genehmigungen, Anhaenge (Bild, Datei), Mentions, Slash-Befehle, Verlauf, Suche, Archiv, Snooze.
- Threads und Projekte: anlegen, umbenennen, pinnen, Drag-and-drop, Worktrees, Branches, Umgebungen (lokal und entfernt).
- Composer: Rich-Text, Code-Bloecke, Listen, Chips, Einfuegen, Undo/Redo, IME.
- Diff-Panel, Dateibaum, Dateivorschau (Text, Bild, PDF, Office, HEIC/TIFF).
- Git: Status, Commit, Push, PR erstellen, PR-Detail, Quellkontroll-Anbieter.
- Terminal: mehrere Terminals je Thread, Resize, Scrollback, Schrift.
- Preview-Browser: Server-Runtime, Mini-Player, Schaerfe, Eingaben.
- Einstellungen: alle Seiten, Schrift, Theme, Kontrast, Zoom, Tastaturkuerzel, Benachrichtigungen (Lautstaerke), Verbindungen, Geraete, Pairing (Handy-App), Diagnose.
- Usage-Anzeigen (verbraucht in Prozent).
- Benachrichtigungen: In-App, Alert-Karte ueber allen Fenstern, Klick oeffnet den Chat, Badge.
- Desktop: frameless Fenster, Ecken und Rand `#252525`, Titelleiste und Steuerknoepfe, Drag, Maximieren, Deep Links (`t3code:`), Single-Instance, CLI-Installer, Snapshots.
- LAN-Endpunkt und Pairing mit der mobilen App.
- Start-/Beenden-Verhalten ohne Waisenprozesse.

---

## 13. Zeitplan (Richtwert, jede Phase einzeln abnehmbar)

| Phase | Inhalt                                 | Dauer          |
| ----- | -------------------------------------- | -------------- |
| 0     | Messwerkzeug und Basis                 | 1 bis 2 Tage   |
| 1     | Golden Suite und Kataloge              | 3 bis 5 Tage   |
| 2     | Server abspecken                       | 3 bis 6 Tage   |
| 3     | Prozess-Struktur (Host, Shell, Orphan) | 2 bis 4 Tage   |
| 4     | Webview-Spike und Entscheidung         | 3 bis 5 Tage   |
| 5     | Web-UI beschleunigen                   | 5 bis 10 Tage  |
| 6     | Packaging und Startzeit                | 2 bis 3 Tage   |
| 7     | Native UI (nur nach Freigabe)          | mehrere Monate |

Gesamt ohne Phase 7: ca. 3 bis 5 Wochen reine Arbeitszeit. Phasen 2 und 3 liefern voraussichtlich den groessten Teil des RAM-Gewinns, Phase 4 und 5 den groessten Teil des Leistungsgewinns.

---

## 14. Arbeitsregeln fuer die Umsetzung

- Branch: `cc3-performance`. Direkter Push auf diesen Branch, kein Pull Request, solange nicht ausdruecklich verlangt.
- Vor jedem Push: Web-Typecheck (`pnpm vp run --filter @t3tools/web typecheck`). `vp run check:changed` stuerzt in diesem Repo ab (Stack Overflow in `scripts/check-changed.ts`); das ist als eigener Punkt vor Phase 1 zu beheben.
- Pre-Commit fuehrt `vp fmt`, `vp lint` und knip aus (ungenutzte Exporte fuehren zum Fehlschlag).
- `project-resume.md` bei jedem abgeschlossenen Teilschritt aktualisieren.
- Messwerte gehoeren in `docs/perf/`, nicht in Commit-Nachrichten allein.
- Jede Aenderung nennt in der Commit-Nachricht: Ziel, Messwert vorher/nachher, Ergebnis der Golden Suite.

---

## 15. Entscheidungen des Nutzers (verbindlich)

1. Phase 4: CEF/Chromium statt WebKitGTK wird uebernommen, sobald der Spike mindestens 30 Prozent Gewinn zeigt.
2. Phase 3.1: Der Node-Host darf in den Server-Prozess integriert werden.
3. Golden-Suite-Referenzbilder liegen im Repo.
4. Phase 7 (native UI) ist **Pflicht** und der Hauptfokus. Dafuer wird bewusst sehr viel Zeit eingeplant, damit jeder Effekt, jede Animation und jedes Pop-up identisch bleibt. Das Entscheidungstor aus Abschnitt 10 entfaellt; Phasen 0 bis 6 laufen weiter als Basis und als Rueckfallebene (`--ui=web|native`).
5. Branch: `cc3-performance`, direkter Push, keine Pull Requests.
