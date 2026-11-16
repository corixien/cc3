# Native UI (Phase 7): Architektur und Spike

Ziel: CC3 ohne Browser-Engine und ohne JavaScript, optisch und funktional 1:1 wie die Web-UI. Dieses Dokument haelt die Architekturentscheidung und die Messwerte des ersten Spikes fest. Code: `apps/cc3-native/` (eigenes Cargo-Workspace, die Tauri-Shell unter `apps/cc3-desktop` bleibt unberuehrt).

## Entscheidung: HTML/CSS-Renderer in Rust (Blitz) statt eigenes Toolkit

Die UI wird weiterhin als DOM mit **denselben CSS-Klassen und demselben gebauten CSS** (Tailwind-Ausgabe von `apps/web`) beschrieben, aber von **Blitz** gerendert, nicht von einem Browser:

| Schicht        | Bibliothek                                                       | Aufgabe                                                                          |
| -------------- | ---------------------------------------------------------------- | -------------------------------------------------------------------------------- |
| CSS-Auswertung | Stylo (Firefox-CSS-Engine)                                       | Kaskade, Variablen, `calc`, `color-mix`, Selektoren, Transitions und Animationen |
| Layout         | Taffy                                                            | Flexbox, Grid, Block                                                             |
| Text           | Parley (+ system fonts)                                          | Schriftform, Umbruch, Lexend Deca und Liberation Mono                            |
| Zeichnen       | Vello (GPU) / Vello CPU (Tests)                                  | Pfade, Verlaeufe, Schatten, Clips, Bilder, SVG                                   |
| Fenster/Events | winit + blitz-shell                                              | Eingabe, Fokus, Zugaenglichkeit (accesskit)                                      |
| Logik          | Rust (Dioxus-native als Komponentenschicht, noch zu bestaetigen) | Zustand, Events, Datenanbindung an den Server                                    |

Gruende:

- Die UI-Aussehensregeln (Farben, Radien, Schatten, Abstaende, Schriften, Hover/Fokus-Zustaende, Animationen) stehen bereits als CSS in `apps/web/src/index.css` und in den Klassenlisten der Komponenten. Sie werden uebernommen, nicht von Hand nachgebaut. Ein eigenes Toolkit (GPUI, Slint, egui) muesste jeden Effekt neu erfinden und kaeme nie auf Pixelgleichheit.
- Die Portierung einer Komponente ist nahezu mechanisch: JSX wird zu Rust-Komponenten mit identischen Klassen-Strings, die React-Logik wird zu Rust.
- Blitz unterstuetzt CSS Transitions und Animationen ueber Stylos Animationsmaschinerie (`blitz-dom/src/stylo.rs`), damit sind die Animationen aus `docs/perf/ANIMATION-CATALOG.md` grundsaetzlich abbildbar.

## Spike-Ergebnis (Messung, 16. Nov 2026)

Ablauf: Der Golden-Test `apps/web/test/golden/dom-snapshot.golden.tsx` exportiert DOM und gesamtes CSS der Seite "thread-history, dunkel" als eigenstaendiges HTML (`apps/cc3-native/snapshots/thread-history-dark.html`, 767 KB). `cc3-native` (Blitz 0.3.0-beta.4, Vello CPU) rendert das ohne JavaScript und ohne Browser nach PNG; `tools/compare.py` vergleicht mit dem Chromium-Referenzbild.

- Renderzeit fuer die ganze Seite: 1,3 s (Parsen, Stylo, Layout, Zeichnen auf der CPU, Release-Build, inklusive Prozessstart).
- Unveraenderter DOM-Snapshot ohne jede Anpassung: **10,38 Prozent der Pixel weichen ab** (Schwelle 8/255 je Kanal), mittlere Abweichung 6,78/255. Das ist der Ausgangswert, den die Portierung auf nahe 0 bringen muss.
- Layout von Sidebar, Kopfzeile, Rechts-Panel, Composer, Nachrichtenblasen und Schriften (Lexend Deca, Liberation Mono) stimmt auf den ersten Blick mit Chromium ueberein.

Bekannte Abweichungen aus dem Diff-Bild (alle behebbar):

1. Inline-SVG-Icons (lucide) fehlen: vermutlich `currentColor` bzw. Stroke-Attribute werden von Blitz nicht an usvg weitergegeben. Zu beheben in Blitz (Beitrag) oder durch Vor-Rasterisierung.
2. Die Nachrichtenliste ist in der Web-UI per JavaScript positioniert (virtualisierte Liste mit absoluten Offsets); der Snapshot enthaelt diese Positionen nicht korrekt. In der nativen UI ersetzt Rust-Logik diese Liste, das Problem entfaellt.
3. Text-Kantenglaettung: Parley/Vello zeichnet Glyphen anders als Skia/FreeType in Chromium. Perfekte Pixelgleichheit ist hier nicht zu erwarten; siehe Metrik unten.
4. Hintergrund-Verlauf der Kopfzeile der Sidebar und Blur-Flaechen: Verlauf weicht ab (Farbraum), `backdrop-filter` muss geprueft werden.

## Metrik fuer "1:1" bei nativem Rendering

Pixelgleichheit bei Text ist zwischen zwei Rasterisierern prinzipiell nicht null. Daher zwei Messungen:

1. **Geometrie:** Position und Groesse aller sichtbaren Elemente (Chromium `getBoundingClientRect` gegen Blitz-Layout) muessen auf 0,5 px uebereinstimmen. Das ist exakt und deterministisch.
2. **Pixel:** Anteil abweichender Pixel (Schwelle 8/255) in Flaechen ohne Text muss 0 sein; mit Text wird eine feste Obergrenze je Zustand festgelegt und nur gesenkt, nie erhoeht.

## Stand N1/N2 (16. Nov 2026, abends)

Werkzeuge:

- Golden-Harness exportiert je Zustand `<name>.html` (DOM plus gesamtes CSS), `<name>.geometry.json` (Box jedes Elements: Tag, x, y, Breite, Hoehe, display, Klasse) und das Chromium-PNG (`GOLDEN_EXPORT=1 pnpm exec vp test run --project golden -u` in `apps/web`, ca. 3 Minuten). Snapshots liegen nicht im Repo (`apps/cc3-native/snapshots/`, 68 MB, per `.gitignore`).
- `cc3-native report <snapshots> <goldens> <out>` rendert alle Zustaende (Vello CPU) und schreibt Pixel- und Geometrie-Vergleich (`report.md`), `DEBUG_GEOM=<name>` listet abweichende Elemente mit Klasse, `CC3_DEBUG_SVG=<teilstring>` gibt die an usvg uebergebene SVG-Quelle aus.
- Golden-Fehler behoben: Das dunkle Theme wurde bisher nicht angewendet (alle "dark"-Bilder waren hell); die Harness ruft jetzt `setTheme`/`setAppearanceMode` auf.

Angepasste Bibliotheken (vendored unter `apps/cc3-native/vendor/`, jede Aenderung mit `CC3:` markiert, `[patch.crates-io]` in `Cargo.toml`):

1. `blitz-dom/serialize.rs`: SVG-Farben (`currentColor`, oklch, color-mix) werden vor usvg nach sRGB `rgba()` umgerechnet; berechnete `fill`, `stroke`, `stop-color`, `stop-opacity` aus der Seiten-CSS (Klassen wie `fill-current`, `var()`) werden als `style` in die SVG geschrieben; `var()`-Deklarationen werden entfernt.
2. `parley_engine/shape/shaped_text.rs`: Glyph-Vorschuebe wie Chromium unter Linux bei 1x (Basisvorschub auf ganze Pixel gerundet, Kerning bruchteilig addiert), Schalter `parley_engine::ROUND_ADVANCES`; gemessen an 196 Buchstabenpaaren und 20 Zeichenketten (15 von 20 exakt, Rest 1 px).
3. `stylo`: `text-box`, `text-box-trim`, `text-box-edge`, `stop-color`, `stop-opacity` auch fuer die Servo-Engine aktiviert (`properties/longhands.toml`, `shorthands.toml`, `shorthands.rs`).
4. `blitz-dom/layout/inline.rs`: `text-box-trim` mit `cap alphabetic` (erste Zeile auf Versalhoehe, letzte auf Grundlinie), Hoehen und Positionen stimmen mit Chromium ueberein.
5. Mitgelieferte Schriften: `assets/fonts` (Lexend Deca, Liberation Mono), per Parley registriert.

Messung (88 Zustaende, Schwelle 8/255): Geometrie je nach Zustand 73 bis 98 Prozent der Elemente auf 0,5 px, Pixel 0,9 bis 13 Prozent abweichend (Text-Kantenglaettung ist der Grundsockel).

Offene Abweichungen (Backlog N2):

- SVG `<pattern>` wird von `anyrender_svg` nicht gezeichnet (stattdessen rote Flaeche): betrifft das Blueprint-Banner im Sidebar-Kopf. Loesung: Kachel-Schleife in `anyrender_svg::render::fill`.
- Text "Search" im Sidebar-Suchfeld, Projekt-Monogramm "RR" und mehrere Avatare fehlen.
- Rauschen/Korn-Overlay der Seite (Chromium-Hintergrund hat 3 bis 5 Stufen Rauschen).
- `overflow: hidden` schneidet getrimmte Texte (Unterlaengen) ab; Chromium schneidet nicht.
- Positionen langer Texte um 1 px (Kerning-Reste), Scroll-Container (Timeline-Liste) kommen aus JS.

## Arbeitspaket-Reihenfolge

1. **N0 (erledigt):** Spike, Renderer ohne Fenster, Vergleichswerkzeug.
2. **N1:** Messwerkzeug erweitern (Geometrie-Export aus Chromium, Vergleich mit Blitz-Layout, Pixel-Diff je Zustand fuer alle Golden-Bilder, Bericht als Tabelle).
3. **N2:** SVG-Icons, Verlaeufe, Schatten, Blur: Abweichungen aus den 138 Referenzbildern auf Blitz-Fehler zurueckfuehren und beheben (Patches oder Beitraege an Blitz).
4. **N3:** Fenster und Shell: winit-Fenster mit abgerundeten Ecken und `#252525`-Rand, eigene Titelleiste, Drag, Maximieren, Single-Instance, Deep Links, Badge, Alert-Karte (heutige Logik aus `src-tauri/src/*.rs` wird uebernommen, kein Webview mehr).
5. **N4:** Datenanbindung: Server-Client (WebSocket/RPC, Contracts aus `packages/contracts`) in Rust; Zustandsspeicher analog zu `apps/web/src/state`.
6. **N5 bis N9:** Komponenten nach Schwierigkeit (siehe unten), jeweils mit Golden-Test und Funktionstest vor Ersetzung der Web-Version.
7. **N10:** Komplexe Teile: Markdown-Rendering, Diff-Ansicht, Composer-Editor, Terminal (libghostty), Dokumentvorschau, Vorschau-Browser.
8. **N11:** Umschalter `--ui=web|native`, Paketierung, Messung gegen die Budgets.

## Umfang (aus dem Quellcode gezaehlt)

- `apps/web/src`: 668 TSX-Komponenten, ca. 317 000 Zeilen TS/TSX, 2 700 Zeilen CSS, 11 `@keyframes`, Animationsklassen in 171 Dateien.
- Groesste Einzeldateien: `ChatView.tsx` 4 085 Zeilen, `chat/ChatComposer.tsx` 3 846, `Sidebar.tsx` 3 662, `CommandPalette.tsx` 3 080, `settings/SettingsPanels.tsx` 2 376, `ComposerPromptEditorTiptap.tsx` 2 009, `ThreadTerminalDrawer.tsx` 1 956, `ChatMarkdown.tsx` 1 924.
- Komponentenordner: auth, chat, cloud, desktop, device, diffs, files, guest, mcp, media, onboarding, permissions, preview, pullRequest, search, settings, sidebar, terminal, ui, usage.

## Risiken

| Risiko                                                                                | Gegenmassnahme                                                                             |
| ------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| Blitz ist Beta (0.3.0-beta.4), Luecken bei CSS (z. B. `backdrop-filter`, SVG-Details) | Fruehe Pruefung in N2, Patches oder Fork, Beitraege upstream                               |
| Text-Rasterisierung weicht von Chromium ab                                            | Dual-Metrik, Obergrenzen je Zustand                                                        |
| Rich-Text-Editor (Tiptap/ProseMirror) hat kein Rust-Gegenstueck                       | eigener Editor auf Parley; IME und Auswahl sind Pflicht                                    |
| Upstream-Aenderungen der Web-UI                                                       | Skript, das geaenderte Komponenten markiert; Klassen-Strings bleiben die gemeinsame Quelle |
| Umfang (mehrere Monate)                                                               | Umschalter `--ui=web                                                                       | native`, bildschirmweise Ablaesung, Web-UI bleibt Rueckfall |

## Nutzung des Spikes

```bash
# Snapshot erzeugen (apps/web)
pnpm exec vp test run --project golden dom-snapshot
# Rendern und vergleichen (apps/cc3-native)
cargo build --release
./target/release/cc3-native snapshots/thread-history-dark.html /tmp/native.png
python3 tools/compare.py /tmp/native.png ../web/test/golden/__screenshots__/thread-history-dark.png /tmp/diff.png
```
