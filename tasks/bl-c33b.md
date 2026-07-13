+++
title = "epic: child step loop — dispatched children run to terminal and deposit (§2.5 live)"
created = 1783917121
updated = 1783917121
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["epic"]
+++
## Scope
The single biggest gap to a usable platform: worker.rs stops at the dispatch commit — no model call, no tool loop, no terminal event. Give a dispatched child the same step loop a root runs, so every structurally-wired-but-dormant piece goes live: result deposits (inbox::deposit_child_result), epitaphs, work-product transfer on delivery, parent revival, the died-sweep against real crashed children.

## Design direction (draft in this ball's worktree)
- One loop, not two: the child executor runs run_exchange with the parent-derived deposit target. deposit_child_result is already total (no-op for roots); the gap is the early stop in worker.rs, not a missing loop.
- Child on-ramp needs no goal deposit: the child forks off the commit where the dispatch landed (§2.5), so its inherited transcript ends with the dispatch tool_result — a user-role wire block — and the goal is pinned (§2.8). Wire-valid step 1 with zero new mechanism. Pin this in ARCH §2.5.
- Detached spawn: the dispatch built-in must spawn the child executor to outlive the parent's tool subprocess — own pgid (setpgid, consistent with §2.9 agent-scoped stop), stdio to log/null. Same detachment contract bl-4684 pins for advance; share it.
- Budgets: max_depth and whole-tree token/wall ceilings (§6) already derive against the root prefix; verify they enforce in the child loop unchanged.

## Dependencies
Soft on bl-4684 (advance): deposits land without it, but revival of a quiescent parent waits for the launcher. Do not block claim; note the seam.