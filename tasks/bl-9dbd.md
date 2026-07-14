+++
title = "epic: real compaction — model-driven compactor, checkpoint triggers, the compaction merge (§2.6–§2.7)"
created = 1783917126
updated = 1784010070
claimant = "Reappear-9dbd"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["epic"]

[[blockers]]
id = "bl-c33b"
on = "claim"
+++
## Scope
The compactor is the v0.3 stub: no model call, mark_for_deletion a no-op, and it still runs as *terminal* compaction — a shape the spec deleted (§2.7: "There is no terminal compaction stage anymore"). Land the real thing: a compactor is an ordinary child agent (needs the child step loop, hence the claim blocker) with a real model call via bz, the two-tool toolset (write_summary, mark_for_deletion — deletions applied at commit time, deletion-only structural), checkpoint triggers from workflow.yaml (every_n_commits, every_t_seconds, on_flush), and the compaction merge: the dispatching agent's executor lands the compactor's branch --no-ff at a step boundary, live-branch-wins on work-product deletions, cache rebuild point (§5.5). Delete the terminal-compaction call site.

## Must-pin items
- Toolset shape: write_summary/mark_for_deletion as in-process built-ins available only to the compactor role (§2.7 says built into the primitive, not declared in providers.yaml).
- Trigger evaluation home: commit-count/elapsed triggers are read at step boundaries by the executor (workflow config is already loaded there); the flush action is the agent-elected trigger.
- Checkpoint commit C derivation and the disjoint-write-set argument (§2.6) hold only under transcript immutability — test the one overlap (compactor nominates a work product rewritten since C; executor drops that deletion).
- Compaction failure lands no merge; branch continues uncompacted (§2.7).