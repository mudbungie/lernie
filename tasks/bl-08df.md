+++
title = "Design: terminology ladder v2 — workspace / config / agent; exchange demoted to span"
created = 1783819503
updated = 1783819503
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["design-2026-07"]

[[blockers]]
id = "bl-1c94"
on = "close"

[[blockers]]
id = "bl-f91a"
on = "close"

[[blockers]]
id = "bl-97fb"
on = "close"
+++
Settled in design discussion 2026-07-09..11. Deliverable: living rewrite of ARCH §2.1 (terms table + banned list) and the lernie-stance paragraphs in TAXONOMY (§1 step/turn notes, §3 conversation/thread/session sprawl).

The ladder: WORKSPACE = the isolation boundary, one repo (no cross-workspace mechanism exists — structural no-cross-talk). CONFIG = the contents of one configuration commit (souls, tool enablements, grants, workflow, manifest); a descriptively-named config branch is the version lineage, its head the current config — 'multiple configs' = multiple branches; there is no main. AGENT = one living instantiation: goal, growing context, step loop, termination; substrate = branch + worktree forked off a ref (default: head of a config branch). EXCHANGE = the span between a user message and the terminal response — UX label over the agent's linear history, owns no branch/merge/lifecycle. STEP, MODEL CALL, ATTEMPT unchanged from current §2.1.

Retirements/decisions to record: 'conversation' and 'conversation repo' retire as structural terms; 'subagent' deletes as a category (parent/child is provenance only); 'session' STAYS banned (transport + per-framework overload); 'thread' was considered and rejected (superseded by agent-as-instance; also collides with §3.1 'Threads, not processes'); 'agent' is pinned to the running-instance sense (field consensus: an LLM running tools in a loop toward a goal), NOT the config sense — the config sense is named 'config'. Resolve the workspace-vs-work-products collision: 'workspace' = the repo-level boundary; in-worktree non-transcript files are 'work products'.