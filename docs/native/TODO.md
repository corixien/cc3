# CC3 native: Rest bis 1:1 mit Tauri-Version

Referenz: Chromium-Goldens in `apps/web/test/golden/__screenshots__`, Messung `cc3-native ui-report`. Ziel je Zustand: Pixel < 2 %, Geometrie > 95 %. Haken setzen, wenn Block fertig und committed.

## A. Settings an Server binden (`ui/settings.rs`, `store.rs`)

- [ ] General, Appearance (Theme, Schrift, Zeitformat), Chat, Diffs, Browser, Voice, Beta, Labs: Werte aus Server-Settings, Schalter/Inputs/Selects schreiben per RPC zurueck
- [ ] Providers (Status, Modelle, Login), Connections, Devices, Access, Secrets
- [ ] Keybindings (Liste, Editieren, Aufnahme)
- [ ] Projects, Threads, Archived (Wiederherstellen), Worktrees, Storage, Maintenance, Diagnostics, Scheduled tasks, Source control, Integrations, About, Licenses

## B. Timeline-Zeilen (`ui/timeline_rows.rs`, `model.rs`)

- [x] Work-Summary "Worked for", Gruppe und Eintragsliste (Reasoning, command, file_change, web_search) aufklappbar; Einzeleintrag-Detail (Output, Diff) offen
- [ ] Diff-Zeilen in Timeline, proposed_plan, todo_list, approval_request, checkpoint (user_input_request: Frage beantworten fertig)
- [ ] Code-Bloecke mit Syntax-Highlighting, Tabellen, Links, Bilder, Copy-Button
- [ ] Streaming-Updates, Auto-Scroll, Scroll-to-bottom-Button, Minimap-Scroll

## C. Overlays (`ui/overlays.rs`)

- [x] Command-Palette (Ctrl+K, Suche, Pfeiltasten, Enter, Esc); Aktionen ausser Page-Wechsel sind Platzhalter
- [ ] Kontextmenues (Sidebar-Zeile: Rename, Archive, Delete, Pin), Hover-Zeile
- [ ] Tooltips, Toasts (Queue, Timeout, Animation), Dialoge (Bestaetigung)
- [ ] Globale Tastenkuerzel laut Keybindings

## D. Panels (`ui/panels.rs`)

- [x] Diff-Panel (Dateiliste, Unified, Zeilenfarben, Refresh); Split, Hunks, Stage/Revert offen
- [x] Terminal (PTY ueber Server, vt100-Screen, Farben, Tastatur); Tabs, Resize, Maus offen
- [ ] Dateibaum/Files, Preview

## E. Composer (`ui/composer.rs`)

- [x] Modell-Picker (Liste, Suche, Auswahl wirkt auf neue Nachrichten); Effort, Modus (Plan/Build), Runtime-Mode offen
- [ ] @-Mentions, Slash-Commands, Anhaenge/Bilder einfuegen, Queue-Nachrichten, Stop/Interrupt
- [ ] Mehrzeilen-Hoehe, IME, Undo

## F. Renderer (`vendor/blitz-*`, `render.rs`)

- [ ] backdrop-filter blur, transform-bewusste Geometrie, Inline-input-Baseline, scrollbar-gutter
- [ ] Animationen (Transitions, Keyframes) wie anim-* Goldens

## G. Weitere Seiten

- [x] Welcome (Start oeffnet Projekt-Dialog; erscheint ohne Projekte); Pull requests (Daten), Usage (Daten), Cloud, Devices, New chat: Entwurf + erste Nachricht erstellt Thread (fertig); Projektwahl, Branch/Worktree-Toolbar offen
- [ ] Sidebar: Projekte gruppieren, Suche, Sortierung, Collapsed-Zustand, Resize

## H. Betrieb

- [x] `--ui=native` Schalter im Tauri-Start (exec cc3-native-ui oder CC3_NATIVE_BIN), strip im Release-Profil; Packaging, RAM/CPU-Messung, Server-Orphan bei SIGTERM offen
- [ ] Start der App erst bei ~99 % Paritaet
