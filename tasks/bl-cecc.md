+++
title = "scripts/pre-commit collapses to exec bl-gate (userconf); gate wording in AGENTS.md"
created = 1790735968
updated = 1790735969
claimant = "Junketing-collapse2"
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
ops bl-3166 (phase 2 rollout). The gate body lives once in ~/userconf/bin/bl-gate. The seated .githooks/pre-commit (mainline refusal, execs scripts/pre-commit) stays; scripts/pre-commit becomes `exec bl-gate "$@"`. make check is already the whole gate (bl-7336); nothing to fold.