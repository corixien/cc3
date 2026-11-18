@AGENTS.md

# CC3 rules

- Always work on the branch the user specifies. Never switch branches or commit to another one on your own.
- This Electron app is a fork of upstream t3code (`upstream` remote = pingdotgg/t3code, `origin` = corixien/cc3). It must stay mergeable with upstream updates: `git fetch upstream && git merge upstream/main`.
- Our changes must never be overwritten by an upstream merge. On conflicts keep CC3 behavior and branding and take the upstream change around it.
- To keep merges cheap, put CC3 features in new files (for example `apps/desktop/src/cc3*.ts`) and keep edits to existing upstream files small.
- Scope: Linux desktop app and Android app only. No iOS, macOS, Windows or marketing site.
- Branding: the product name is "CC3" (not "T3 Code"); the CC3 logo with outline is used for every icon.
