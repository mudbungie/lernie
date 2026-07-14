+++
title = "epic: skill body-on-demand — agent-elected skill load into the worktree (§3.3)"
created = 1783917134
updated = 1784009490
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["epic"]
+++
## Scope
Descriptions-always is shipped (descriptions/** snapshot, composed every model call); Body-on-demand is not: no mechanism exists for an agent to elect a skill and get its body into context. Land §3.3's contract: on election, the harness copies <data-root>/skills/<name>/ into the branch worktree at skills/<name>/, committed like any tool side effect; the next assembly composes it (manifest order category skills/**, §5.2); the compactor may prune it later; copy-not-symlink for portability.

## Design direction (draft in this ball's worktree)
- Mechanism: a built-in tool — lernie tool load_skill, input {name} — is the structural fit: the election is a tool call, so it lands as a transcript entry + worktree commit under the existing commit-per-side-effect discipline (§3.3), auditable and replayable with zero new channel. The alternative (agent bash-copies from the data root) leaks the data-root path into souls and bypasses nothing usefully.
- Unknown skill name → is_error tool_result naming the available pool (decline, don't guess).
- Already-loaded skill → idempotent success (the copy is the state; re-copy is a no-op if identical, error if the pool changed? — no: snapshot discipline says the loaded copy wins; report already-loaded).
- Cache pricing: a mid-run load inserts into the body and flushes the provider cache from its position (§5.5 already prices this); the SKILL.md description should say so ("load early").
- Tool name is an open decision (user): load_skill vs skill. Recommend load_skill — "skill" the noun collides with the primitive in prose and prompts.