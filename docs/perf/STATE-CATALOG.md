# Zustandskatalog (Phase 1.1)

Referenzbilder: `apps/web/test/golden/__screenshots__/` (Chromium, 1400x900, eingefrorene Uhr, Animationen auf Endzustand, Toleranz 0 Pixel).
Aufnehmen: `pnpm exec vp test run --project golden -u` (in `apps/web`). Vergleichen: ohne `-u`.
Test: `apps/web/test/golden/app.golden.tsx`. Umgebung: `test/fakeEnvironment.ts` (kein echter Server).

## Abgedeckt (je dunkel und hell)

- thread-history: Thread mit Verlauf
- thread-question: Thread mit offener Frage (Banner am Composer)
- new-chat: leerer neuer Chat
- Einstellungsseiten (aus `routes/_chat.settings.*.tsx`):
  - settings-about
  - settings-access
  - settings-appearance
  - settings-archived
  - settings-beta
  - settings-browser
  - settings-chat
  - settings-connections
  - settings-devices
  - settings-diagnostics
  - settings-diffs
  - settings-general
  - settings-integrations
  - settings-keybindings
  - settings-labs
  - settings-maintenance
  - settings-open-source-licenses
  - settings-projects
  - settings-providers
  - settings-scheduled-tasks
  - settings-secrets
  - settings-snap-shot
  - settings-source-control
  - settings-storage
  - settings-threads
  - settings-voice
  - settings-worktrees

## Routen ohne eigenes Referenzbild (TODO)

- approve-agent
- _chat.$environmentId.$threadId
- _chat.draft.$draftId
- _chat.index
- _chat.new
- _chat.pull-requests
- _chat.settings
- _chat
- _chat.usage
- connect-agent
- connect
- guest
- pair
- projects.$projectKey
- __root
- threads.$environmentId.$threadId
- welcome

## Zustaende, die noch fehlen (TODO, nach Komponentenordner in apps/web/src/components)

- chat: Rate-Limit-Banner, Provider-Status, Thread-Fehler, Genehmigungen, Plan-Karten, Tool-Aufrufe, Anhaenge, Streaming-Zustand
- Sidebar: Hover, Auswahl, Drag, Snooze, Gruppen, gepinnt, viele Threads
- CommandPalette: offen, Suche, Treffer
- DiffPanel, Dateibaum, Dateivorschau (Text, Bild, PDF, Office)
- terminal: Drawer offen, mehrere Terminals
- preview: Mini-Player, Resize-Griffe, Vorschau-Fenster
- usage: Limits-Leiste, Pool-Details, Thread-Details
- Dialoge, Popups, Toasts, Kontextmenues, Tooltips (ui/)
- onboarding/WelcomePage, auth, pair, connect, guest, cloud, device, desktop
- Fenster: frameless Ecken und Rand (nur in der Tauri-Shell pruefbar), Alert-Karte (alert.rs)
- weitere Themes, Kontrast- und Schrifteinstellungen, Zoomstufen
