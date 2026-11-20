#!/usr/bin/env bash
# CC3: list upstream (pingdotgg/t3code) bug-fix and security commits CC3 does not have yet, so
# small patches can be cherry-picked without merging upstream's bigger feature changes.
#   scripts/cc3-upstream-patches.sh            list fixes/security since the last merge
#   git cherry-pick -x <sha>                   take one (resolve conflicts in favor of CC3)
set -euo pipefail
git fetch upstream --quiet
git log --no-merges --reverse --date=short \
  --pretty='%h %ad %s' \
  -i -E --grep='^(fix|security)' --grep='CVE|vulnerab|security' \
  HEAD..upstream/main
