+++
title = "bundle/replay: replayed workspace cannot be driven — no config/* ref survives"
created = 1784955704
updated = 1784958889
claimant = "Osprey"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Foxglove finding 2, release-critical. The §9.2 archival story is void: agents.bundle carries only refs/heads/agents/* — no config/* refs — while governing-config derivation is merge-base against the config/* HEADS. Verified: LERNIE_HOME=iso lernie prompt <replay> → 'git rev-parse refs/heads/config/default … fatal: Needed a single revision'; lernie advance → 'no config/* ancestor for agents/<id>'. Worse, replay's attempted delivery committed the inbox message before failing, leaving delivered-but-unanswered. Fix per ARCH §9.2/§2.2 (read both, decide the minimal true design): most likely lernie bundle must include the config/* refs that are ancestors of the subtree (the governing lineage), so the replayed repo derives exactly as a real one. Do not invent a sidecar. Tests: bundle→replay→prompt round-trip drives green.

## Status 2026-07-24 (Osprey)

Implementation COMPLETE and verified in worktree work/bl-aabb (commit "bundle: carry the governing config lineage so a replay is drivable"), main merged through 8d8638c. Fix: `workspace::config_lineage` (the `config/*` refs a merge-base against the subtree root resolves) is added to the bundle's ref list; `bundle_heads` now derives the primary over agent refs only. Regression proven both ways: with the lineage suppressed the two new e2e cases fail with exactly the reported errors; with it they pass. Delivery ordering left as-is (deliver-before-resolve) — a delivered message is a committed transcript entry and warrant re-derives it as ModelCallDue, so the branch is re-drivable, not stranded; reasoning recorded in ARCH §9.2.

CLOSE BLOCKED on bl-8c92 (brazen pin): machine `bz` is 0.0.4, this tree pins =0.0.3, so the load-time guard fails all seven real-bz e2e tests in the close gate (`bz version "0.0.4" does not match the linked brazen crate "0.0.3"`). With a local =0.0.4 pin the whole suite passes (839/840; the single miss was the bl-1c2e flake, since fixed on main). Close as soon as bl-8c92 lands on main and is merged here.