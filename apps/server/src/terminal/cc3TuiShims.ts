// CC3: helper commands the Claude TUI gets on its PATH, plus the instructions that tell Claude about them.
// They talk to CC3 through an OSC sequence on the TTY (client side) or the terminal history files.
export const CC3_TUI_SHIMS: Readonly<Record<string, string>> = {
  "xdg-open": String.raw`#!/bin/sh
printf '\033]777;cc3-open;%s\007' "$1" > /dev/tty
`,
  "cc3-terminal": String.raw`#!/bin/sh
[ $# -gt 0 ] || { echo "usage: cc3-terminal <command>" >&2; exit 2; }
printf '\033]777;cc3-term;%s\007' "$(printf %s "$*" | base64 -w0)" > /dev/tty
echo "Command typed into CC3's terminal. It is not run until the user presses Enter. Read the result with cc3-terminal-output."
`,
  "cc3-open-file": String.raw`#!/bin/sh
[ $# -gt 0 ] || { echo "usage: cc3-open-file <path>[:line]" >&2; exit 2; }
case "$1" in /*) p="$1" ;; *) p="$PWD/$1" ;; esac
printf '\033]777;cc3-file;%s\007' "$(printf %s "$p" | base64 -w0)" > /dev/tty
echo "Opened in CC3's file viewer."
`,
  "cc3-terminal-output": String.raw`#!/bin/sh
b64() { printf %s "$1" | base64 -w0 | tr '+/' '-_' | tr -d '='; }
h="$HOME/.t3"; [ -n "$T3CODE_HOME" ] && h="$T3CODE_HOME"
n=60; [ -n "$1" ] && n="$1"
f=$(ls -t "$h"/*/logs/terminals/terminal_$(b64 "$CC3_TUI_SESSION")_$(b64 term-1).log 2>/dev/null | head -1)
[ -n "$f" ] || { echo "CC3's terminal has no output yet."; exit 1; }
tail -c 30000 "$f" | LC_ALL=C sed -e 's/\x1b\[[0-9;?<>=]*[ -\/]*[@-~]//g' -e 's/\x1b\][^\x07]*\x07//g' | tr -d '\r' | tail -n "$n"
`,
};

export const CC3_TUI_SYSTEM_PROMPT = `You are running inside CC3, a desktop app that shows you as a terminal next to a built-in browser, file browser and a separate terminal drawer. Cooperate with it:
- To show a website or open a URL, use the built-in browser: the mcp__cc3__preview_open tool with \`open: true\` (so the browser panel comes to the front), then preview_navigate/snapshot as needed. Never launch an external browser.
- When a command needs the user (sudo, passwords, interactive prompts), do not run it yourself: run \`cc3-terminal "<command>"\`, which types it into CC3's terminal drawer without running it, and ask the user to press Enter. Afterwards read what happened with \`cc3-terminal-output [lines]\`.
- To show the user a file (code, document, image, video, PDF, presentation), run \`cc3-open-file <path>\`: it opens in CC3's file viewer. Do this whenever you want the user to look at a file.
- Everything else you run through your normal shell tools.`;
