+++
title = "seed the agent's working directory at spawn: --cwd on lernie prompt and lernie dispatch"
created = 1785823804
updated = 1785996510
claimant = "Tubercle"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["upstream-ask"]
+++
Source: yog bl-2b8c (closed 2026-08-03; yog docs/VISION.md §4.10 is the cross-suite authority). Premises re-verified against the tree 2026-08-05 and rescoped with the user the same day.

## Contradiction

A consumer (yog) that wants an agent to work in an external checkout has no mechanical channel: the target rides goal prose the model must choose to obey. The fact home already exists — the working-directory mark `refs/lernie/cwd/<agent-id>` (ARCH §3.3, `src/workspace/cwd.rs`), written by the `cd` built-in and read by the executor at every tool spawn (`prompt::tool::spawn::Caller::resolve`). But only the agent can write it, and only once it is already running.

## Why a parameter, and not a caller-side ref write

The mark's format is public, so a caller could write the ref itself with two git commands. It cannot win the race: `child_dispatch::run` deposits the goal and launches the driver in one move (`src/prompt/child_dispatch.rs:205-206`), so the caller learns the agent id only after step 1 may already be underway, and there is no lock to take. Creation is the only moment that precedes the first step. The parameter buys **ordering**, not capability — and keeps consumers out of lernie's ref namespace.

## Deliverable

1. `--cwd <path>` on `lernie prompt` and `lernie dispatch`, seeding the existing mark before the first step. No new fact home and no new channel: a second writer for the one mark the executor already reads. Omission is today's behaviour (the agent's worktree) — the general path with the fact absent, not a bootstrap case.
2. Validated before the fork, the way pins are (ARCH §2.5 — every refusal precedes the fork, so no branch, ref or inbox exists when one fires): the path must exist and be a directory (`builtin::cd::resolve`'s own decline, applied earlier) and must survive the mark's trimmed-UTF-8 round trip (`workspace::cwd::storable`). One voice with `cd`, not a second set of rules.
3. Written after the name/id settles and before the deposit that wakes the driver. A failed mark write fails the dispatch — no agent starts in a directory its caller did not ask for.
4. No inheritance: marks are keyed by agent id and nothing forks, merges or transfers them, so a child's is unset unless its own dispatch names one. Already true in the tree — state it in ARCH §3.3 and pin it with a test, because yog's writer-isolation law (VISION §4.10: no two write-capable lineages share a mutable checkout) depends on absence being the default.
5. Policy-blind: the parameter is a directory. lernie learns nothing of balls, claims, targets, candidates, attempts or delivery.

## Ruled out (2026-08-05)

- **The `dispatch` tool input gets no such parameter.** yog fires through the CLI. Adding it to the model-facing schema costs tokens on every request and hands a running agent the power to place children anywhere on the machine. Add it if and when a consumer needs it.
- **Per-step project provenance is dropped** (was: record the outside repo's HEAD in each step record). It would make lernie run git inside a directory it does not own — a bl worktree, in yog's case — reading another tool's storage as a side effect of recording a step. The consumer knows which checkout it handed over and can record that fact through its own front door.
- **Declining on a vanished marked directory is dropped.** ARCH §3.3 rules the other way and gives the reason: "A mark whose directory has since disappeared resolves to the worktree **rather than declining the call**: `cd` is itself a tool call, so a hard decline would strand the agent somewhere it could never leave." With the path validated at creation the case is rare; reversing a shipped ruling needs its own argument. Behaviour unchanged here.

Downstream: yog consumes this via an exact published pin — its fire passes the claim-derived worktree and the goal-prose location preamble retires.