+++
title = "bundle/replay: replayed workspace cannot be driven — no config/* ref survives"
created = 1784955704
updated = 1784955704
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Foxglove finding 2, release-critical. The §9.2 archival story is void: agents.bundle carries only refs/heads/agents/* — no config/* refs — while governing-config derivation is merge-base against the config/* HEADS. Verified: LERNIE_HOME=iso lernie prompt <replay> → 'git rev-parse refs/heads/config/default … fatal: Needed a single revision'; lernie advance → 'no config/* ancestor for agents/<id>'. Worse, replay's attempted delivery committed the inbox message before failing, leaving delivered-but-unanswered. Fix per ARCH §9.2/§2.2 (read both, decide the minimal true design): most likely lernie bundle must include the config/* refs that are ancestors of the subtree (the governing lineage), so the replayed repo derives exactly as a real one. Do not invent a sidecar. Tests: bundle→replay→prompt round-trip drives green.