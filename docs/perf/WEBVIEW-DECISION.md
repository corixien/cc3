# Webview-Entscheidung (Phase 4)

Frage: Lohnt es sich, WebKitGTK (System-WebView von Tauri) durch Chromium (CEF) zu ersetzen? Regel aus dem Plan: ja bei mindestens 30 Prozent Gewinn.

## Messungen (Stand 16. Nov 2026, dieselbe Maschine, Ruhezustand, gleicher Seiteninhalt: Onboarding bei leerem Home)

RAM, PSS je Prozess (`scripts/perf/measure.mjs`):

|                           | WebKitGTK in CC3 | Chromium 1248 (Playwright, gleiche UI + bridge.js mit Stub-IPC) |
| ------------------------- | ---------------- | --------------------------------------------------------------- |
| Renderer der UI           | 267 MB           | 131 MB                                                          |
| Netzwerkprozess           | 29 MB            | 47 MB                                                           |
| GPU-Prozess               | im Web-Prozess   | 84 MB                                                           |
| Browser-/Embedder-Prozess | Shell 99 MB      | 141 MB (in einer CEF-App waere das die Shell)                   |
| Zygote/Utility            | -                | ca. 50 MB                                                       |
| Summe (ohne Server)       | ca. 395 MB       | ca. 450 MB                                                      |

Ergebnis RAM: Chromium spart beim Renderer, verliert aber durch GPU-, Network- und Browser-Prozess. Netto kein Gewinn, eher +10 bis +15 Prozent. Dateien: `perf-results/webkit-clean-idle.json`, `perf-results/chromium-spike-idle.json` (der Chromium-Lauf enthaelt zwei zusaetzliche leere Renderer von Playwright, die oben nicht mitgezaehlt sind).

Rendering-Mikrobenchmark (`scripts/perf/webview/bench.html`: 4000 Zeilen mit Schatten, Verlauf und Radius; JS-Schleife; Scrollen per rAF; 300 Relayouts). WebKitGTK 2.54 per `scripts/perf/webview/wk.py` (die echte System-Engine), Chromium per Playwright headless:

| Metrik                                | Chromium    | WebKitGTK | Chromium besser um |
| ------------------------------------- | ----------- | --------- | ------------------ |
| DOM aufbauen und Layout (ms)          | 460         | 911       | 49 %               |
| 300 Relayouts (ms)                    | 1813        | 3712      | 51 %               |
| JS-Schleife (ms)                      | 522         | 507       | 0 %                |
| Frame-Median / p95 beim Scrollen (ms) | 16,7 / 16,7 | 17 / 19   | knapp              |

Einschraenkung: Mikrobenchmark, nicht die echte UI. Chromium lief headless, WebKitGTK in einem sichtbaren Fenster. Die echte UI (Thread-Wechsel, Tippen, Streaming) wurde unter WebKitGTK nicht gemessen.

## Reife von CEF in Tauri

Offizielle Unterstuetzung fehlt; es gibt nur `tauri-apps/cef-rs` (Bindings) und einen experimentellen `tauri-runtime-cef`-Branch. Projekte, die ihn nutzen, vendoren einen eigenen Fork und ein eigenes CLI, weil das Standard-CLI defekte Bundles baut. API-Luecken (z. B. Navigation) sind bekannt. Das Paket waere ca. 250 MB groesser (CEF-Binaries) und der Build braucht den CEF-Download.

## Entscheidung

1. Layout und Rendering sind in Chromium etwa 50 Prozent schneller (Mikrobenchmark), das erfuellt die 30-Prozent-Regel formal.
2. RAM verbessert sich nicht, die Integration ist experimentell und teuer, und der Webview wird in Phase 7 durch die native UI ersetzt.
3. Empfehlung: CEF jetzt nicht integrieren. Wenn die UI-Leistung unter WebKitGTK vor Phase 7 spuerbar stoert, ist CEF der naechste Schritt; Zwischenloesung sind die Web-Optimierungen aus Phase 5, die beiden Engines zugute kommen.
4. Offen fuer die Nutzerfreigabe: Falls Chromium-Pixelgleichheit zum Original wichtiger ist als RAM, waere CEF der einzige Weg, die Chromium-Referenzbilder (Golden-Suite) auch in der App 1:1 zu treffen. WebKitGTK-Abweichungen zur Golden-Suite wurden nicht gemessen.
