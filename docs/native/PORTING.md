# Portierung der Web-UI nach Rust (Arbeitsweise und Reihenfolge)

Ziel: dieselbe Oberflaeche und Funktion wie `apps/web`, aber als Rust-Programm (`apps/cc3-native`, Binary `cc3`) mit Dioxus-Komponenten auf Blitz, ohne Browser und ohne JavaScript. Der Server (`apps/server`) bleibt unveraendert.

## Der Vergleichskreislauf (nach jedem Schritt)

1. **Golden-Bild** des Zustands aus der Web-UI (`apps/web/test/golden`, Chromium, Toleranz 0) und **Geometrie** aller Elemente (`snapshots/<name>.geometry.json`).
2. Die Rust-Komponenten rendern denselben Zustand ueber `dioxus-ssr` zu HTML (`src/ui/screens.rs`) und Blitz zeichnet ihn (`cc3-native ui-report <snapshots> <goldens> <out>`).
3. Der Bericht nennt je Zustand: Anteil abweichender Pixel, Geometrie-Treffer (0,5 px) und ob die DOM-Reihenfolge gleich ist. Ein Umbau gilt nur als fertig, wenn die Werte gleich bleiben oder besser werden (zuletzt: Umbau in datengetriebene Komponenten ohne Aenderung, thread-question dunkel 5,39 Prozent und 81,8 Prozent Geometrie, hell 3,21 Prozent).
4. Zusaetzlich fuer das Nutzererlebnis: `cc3` mit demselben Server gegen die Web-UI laufen lassen (Screenshot, `measure.mjs` fuer RAM und CPU).

## Werkzeuge

- `tools/html2rsx.py <body.xml> <slot|class:prefix#n|children>`: wandelt einen Teilbaum des Chromium-DOM (`<name>.body.xml`, Export mit `GOLDEN_EXPORT=1`) in RSX. Klassen und Struktur stammen so 1:1 aus der Web-UI.
- Von Hand: aus dem RSX eine Komponente mit Props und Bedingungen machen (Vorlage: `src/ui/thread_row.rs`, `sidebar_list.rs`). Unterschiede zwischen zwei Instanzen (aktive Zeile) per Diff der beiden DOM-Teilbaeume finden.
- `ui::app_css()`: das gebaute Tailwind-CSS der Web-UI (`assets/app.css`), `:root` wird zu `.cc3-root`, damit `dark` auf einem Wrapper sitzen kann.

## Stand der Portierung

| Teil                                                              | Status                                                |
| ----------------------------------------------------------------- | ----------------------------------------------------- |
| Fenster, Titelleiste, Ecken, Rand, Resize                         | fertig                                                |
| Single-Instance, Links, Badge, Alert-Karte                        | fertig                                                |
| Server starten, Auth, WebSocket-RPC                               | fertig                                                |
| Shell-Modell (Projekte, Threads), Sidebar-Zeilen mit echten Daten | fertig (Aktualisierung per Stream fehlt)              |
| Layout (Sidebar, Kopf, Chat-Spalte, Composer, rechtes Panel)      | Geruest aus dem Golden-DOM, Chat-Spalte noch statisch |
| Timeline (Nachrichten, Markdown, Tool-Aufrufe, Diffs)             | offen                                                 |
| Composer (Editor, Mentions, Anhaenge, Senden)                     | offen                                                 |
| Einstellungen (ca. 27 Seiten)                                     | offen                                                 |
| Command Palette, Dialoge, Menues, Toasts, Tooltips                | offen                                                 |
| Diff-Panel, Dateibaum, Dateivorschau, Terminal, Vorschau-Browser  | offen                                                 |
| Onboarding, Cloud, Geraete, Gast, Pull Requests                   | offen                                                 |

## Reihenfolge der naechsten Schritte (nach Nutzen)

1. Zustandsspeicher: Shell-Strom (`orchestration.subscribeShell`) und Thread-Strom (`orchestration.subscribeThread`) in Rust projizieren (Vorlage `packages/client-runtime/src/state`, `apps/web/src/state`); Wiederverbindung mit neuem Ticket.
2. Chat-Spalte mit echten Daten: Kopfzeile, Timeline-Zeilen (Benutzer, Assistent, Arbeitsprotokoll), Markdown (pulldown-cmark plus Syntax-Hervorhebung), Banner.
3. Composer: Texteingabe mit Cursor, Auswahl, Undo, IME, Senden (`orchestration.dispatchCommand`), Modellauswahl, Anhaenge, Mentions.
4. Navigation und Thread-Wechsel, neuer Thread, Projekte hinzufuegen, Kontextmenues.
5. Einstellungen und Dialoge (gleiche Bausteine wie `apps/web/src/components/ui`).
6. Command Palette, Suche, Tastaturkuerzel (`packages/shared/src/keybindings.ts`).
7. Diff-Panel, Dateibaum, Dateivorschau, Terminal (libghostty), Vorschau-Browser.
8. Rest (Onboarding, Cloud, Geraete, Pull Requests) und Umschalter `--ui=web|native`, Paketierung.
9. Immer parallel: Golden-Zustaende fuer jede neue Komponente, Animationen aus `docs/perf/ANIMATION-CATALOG.md`, Messung RAM und CPU.

## Stand der Vergleiche (17. Nov 2026, nach Composer und Minimap)

| Zustand                       | Pixel abw.          | Geometrie    | DOM-Reihenfolge       |
| ----------------------------- | ------------------- | ------------ | --------------------- |
| thread-question dunkel / hell | 5,41 / 3,23 Prozent | 84,3 Prozent | gleich (351 Elemente) |
| thread-history dunkel / hell  | 5,94 / 4,31 Prozent | 42,5 Prozent | gleich (551 Elemente) |

Erledigt: Composer `Normal` mit echtem `<textarea>` (Enter sendet, Shift+Enter Zeilenumbruch) und Senden ueber `message.dispatch` (`Store::send_message`, geprueft mit `send_probe` gegen den echten Server), Minimap, Timeline-Abstandshalter (Spacer nach Composer-Variante), Scroll ans Ende beim Oeffnen, Gutter-Ersatz per CSS (`EXTRA_CSS`), Titel aus dem Thread. Offen fuer thread-history: Residuen von 0,5 bis 1 px bei Text, Sidebar-Suchfeld 5 px hoeher, Sidebar-Rail 8 px versetzt, Detail-Panel (`aside`) 53 px zu hoch, Banner-Span im Composer 28 px schmaler. Offen im Composer: Cursor-Rendering, Rich-Text/Mentions/Slash-Befehle, Modellwahl, Anhaenge, Undo/IME-Test, Live-Test mit Klick auf einen echten Thread (Test-Home hat keinen).

## Messung (17. Nov 2026, leeres Home, Leerlauf)

|               | CC3 Tauri                                   | CC3 nativ (`cc3`)                        |
| ------------- | ------------------------------------------- | ---------------------------------------- |
| UI-Prozess(e) | Webview 363 MB plus Shell 97 MB             | 127 MB (Shell mit Layout und 535 KB CSS) |
| Server        | 348 MB                                      | 350 MB                                   |
| Summe         | 807 bis 888 MB                              | 477 MB                                   |
| Prozesse      | Shell, Host (bei Bedarf), Server, WebKit x2 | Shell, Server                            |
| CPU Leerlauf  | unter 1 Prozent                             | 1,3 Prozent                              |

Der Server ist nun der groesste Block (Phase 2 des Performance-Plans: Schema-Aufbau 31 MB, Worker-Threads 78 MB).

## Seiten aus dem Golden-Markup (17. Nov 2026, Abend)

`tools/extract_fragments.py` zerlegt die Snapshots in `assets/fragments/<zustand>.{main,overlay}.html` und schreibt `src/ui/fragments.rs`. `AppLayout` setzt Fragmente in `main` (Routen) und als Overlay (Dialoge, Popups); `ui/page.rs` loest Klicks per Hit-Test auf (`data-settings-section`, `aria-label=Close`, Backdrop). `ui/cq.rs` plus `blitz-dom/src/container_queries.rs` ersetzen `@container` durch Attribute (`data-cq-NAME~="ge-PX"`), die nach dem Layout aus gemessenen Breiten gesetzt werden. Navigation in der laufenden App: Settings (alle Seiten ueber die Navigation), Usage, Pull requests, New thread; Klick-Aktionen sind in `page::tests` abgesichert.

Stand (ui-report, dunkel): alle 28 Settings-Seiten, Usage und New chat haben DOM-Reihenfolge gleich, Geometrie 71 bis 98 Prozent, Pixel 1,6 bis 5,7 Prozent (Rest: `backdrop-filter`-Unschaerfe hinter Dialogen, 1-px-Textresiduen). Schwach: Chat-Zustaende mit Popups (Palette 31 Prozent, Kontextmenue, Tooltip, Toast, Panels), weil die Chat-Spalte dort andere Daten hat; Diff- und Terminal-Panel 20 bis 31 Prozent.

Wichtig: Fragmente sind Golden-Daten. Werte der Settings-Seiten (Provider, Keybindings, Projekte ...) sind noch nicht an den Server gebunden und Schalter/Eingaben noch ohne Wirkung. Das ist der groesste offene Block fuer "alle Funktionen".

Offen: Tool-Call-/Diff-/Plan-Zeilen der Timeline (Fixture `threadWithWork` und `thread-golden-work.projection.json` liegen bereit; der Golden-Lauf dafuer haengt noch), Eingabefeld-Hoehe, `backdrop-filter`, Transform in der Geometrie (Minimap-Streifen), Sidebar-Suchfeld 5 px zu hoch (Baseline von Inline-`input`).

## Interaktionstests ohne Fenster

`tests/interact.rs` mountet `ui::root::AppRoot` (ohne `WindowFrame`) in einem `DioxusDocument` und schickt Pointer- und Tastenevents durch Blitz' Eventsystem (`handle_ui_event`). So pruefen Hotkeys (Strg+K), Klicks (Zahnrad, Schliessen) und Tippen (Palette-Filter) ohne echtes Fenster und ohne Eingabe auf dem Desktop des Nutzers. Wichtig: der Test muss `html_parser_provider: HtmlProvider` setzen, sonst bleibt `dangerous_inner_html` leer. Screenshots nach `$CC3_TEST_OUT`. Globale Eingabe-Injektion (uinput) wird nicht benutzt: sie trifft jedes Fenster unter dem Mauszeiger.
