+++
title = "CHANGELOG [Unreleased] misses bullets for 8 of the 20 deliveries in the 0.0.7 window — the bl-0b1f defect recurring"
created = 1786513656
updated = 1786513656
priority = 1
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
`make promote-changelog VERSION=0.0.7` stamps the accumulated `## [Unreleased]` section as the release notes for 0.0.7. That section currently carries 9 bullets against 20 delivery commits on `main` since `v0.0.6` (fd4c42e..919662f), two of which are gate closes and correctly unlisted. **Eight deliveries have no bullet at all**, so releasing now would ship permanently incomplete release notes.

Missing, in `main` order (oldest first):

- `bl-c3c5` — design record: cryptographic agent attestation (`docs/DESIGN_AGENT_ATTESTATION.md`).
- `bl-7935` — ARCH §3.3 and `multi.rs` cited bl-a690, a tracking ball closed unbuilt; the deferral is now a standing position rather than a ball reference.
- `bl-a4d5` — the `dispatch` schema, `skills/dispatch/SKILL.md` and the budget doc comments misstated the parent/child relationship: a deleted terminal-compaction stage, per-agent budgets that are whole-tree, and no hint that a live child is addressable.
- `bl-32c9` — `DESIGN_MCP_BRIDGE.md` §9 cited bl-8925 as a pending follow-up; it was closed unworked.
- `bl-7173` — ARCH §3.3 and `install/tests/toolspec.rs` used bare "per-call" for the `cd` working directory, the sense §2.1 bans.
- `bl-4ae6` — ARCH §6 never stated the `max_depth` boundary, so the off-by-one lived only in `budget/mod.rs`; §6 now carries it and a test pins it.
- `bl-d0b4` — *(has a bullet)*.
- `bl-ec74` — `multi_tool`: the envelope may assert `execution: "parallel"`; `ToolExecutor::execute_all` with a serial default, overridden by `SpawnTool`'s prepare/spawn/finish split.
- `bl-3361` — the bare "per-call" sweep, tree-wide (~15 further sites).

## Why it keeps happening

This is bl-0b1f verbatim (closed 2026-08-03: *"back-fill 24 missing CHANGELOG [Unreleased] bullets"*). The bullet is a convention stated in `CHANGELOG.md`'s own header and in the ball-close ritual, and nothing enforces it — a delivery closes green with no bullet. Docs-only deliveries are the worst case, since `release-plz` attributes no packaged commit to them and generation could not recover them even in principle (the reason `changelog_update = false`, bl-7558).

## Deliverable

1. The eight missing bullets, in delivery order, in the house style (one bullet, imperative, `[bl-xxxx]` trailer).
2. **A guard, so this is the last time.** The obvious shape is a `tests/commit_hygiene.rs` sibling: every `[bl-xxxx]` id in `git log v<last-tag>..HEAD` that is not a gate close must appear in `CHANGELOG.md`'s `[Unreleased]` section. It is the same class of check `commit_hygiene` already runs over commit messages, reading the same two sources of truth. Filing the guard separately is acceptable if the backfill is urgent; leaving it unfiled is not.