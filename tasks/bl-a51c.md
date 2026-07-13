+++
title = "epic: workspace substrate — config branches, agents/* refs, no main (§2.2–§2.3 physical)"
created = 1783917121
updated = 1783917121
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["epic"]
+++
## Scope
The spec's core physical model is not yet the shipped one. Shipped: lernie new scaffolds a per-conversation repo with git init -b main; agent branches are bare (<conv-id>, <parent>-<sub-id>); control files live in the worktree. Target (§2.2–§2.3): one repo per workspace at <data-root>/workspaces/<workspace>/ (bare repo.git), config branches config/<name> with no main, agent refs agents/<agent-id>, worktrees as siblings under agents/, control read from the governing config commit (nearest config/* ancestor), dispatch commit removes control files from the agent tree, config-commit authoring as a harness-assisted user act.

## Must-pin items
- Ref-prefix seam: scan/stop/budget enumeration is currently "every branch except main" — ARCH §8 calls this "the one seam to change when a prefix lands." Land the prefix and flip that seam in the same ball.
- Governing-config resolution: derived from ancestry (git merge-base against config/* heads), never stored.
- Template/scaffold rewrite: lernie new becomes workspace creation + first config-commit authoring (orphan root), snapshotting descriptions/** as today (bl-3092 seam migrates here per §3.3 shipped-state note).
- LERNIE_CONV_REPO/BRANCH semantics unchanged (names are code, §3.3).
- Root id allocation: unique per workspace.

## Open decision (user)
Migration vs clean break for existing conversation repos — §10 promises old versions stay readable, but pre-v1 a clean break (refuse old layout with a clear error) may be the elegant call.