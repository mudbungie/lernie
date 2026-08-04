+++
title = "typed working-directory binding at creation: seed the cwd mark from prompt/dispatch, validate it, record per-step project provenance"
created = 1785823804
updated = 1785823804
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["design", "upstream-ask"]
+++
Source: yog bl-2b8c (the recursive project-delivery contract, closed 2026-08-03; docs/VISION.md §4.10 on yog main is the cross-suite authority). Verify every premise against the current tree before ruling.

## Contradiction

A harness consumer (yog) that wants an agent to work in an external project checkout has no mechanical channel: the target rides goal prose the model must obey. Yet the fact home already exists — the working-directory mark (`refs/lernie/cwd/<agent-id>`), read by the executor at every tool spawn — with exactly one writer, the model-driven `cd` built-in. The consumer cannot seed it, a child cannot be born with it, and nothing joins the external repo's commits to agent history, so project work is absent from step provenance, bundle/replay, and any lawful attempt comparison.

## Deliverable

1. A creation-time working-directory parameter on `lernie prompt`, `lernie dispatch`, and the `dispatch` tool input, which seeds the existing mark before the first step. No new fact home, no new channel: the parameter is a second writer for the one mark the executor already reads. Omission is today's behavior (agent worktree) — the general path with empty inputs.
2. Validation at both ends: creation refuses a path that is missing or not a directory (the cd tool's own decline, applied earlier); an executor read whose marked directory has vanished declines the tool call loudly and never silently falls back to the agent worktree.
3. No silent inheritance: the mark is id-scoped and a child's is unset unless its dispatch names one. State and test this as contract — the consumer's writer-isolation law (yog VISION §4.10: no two write-capable lineages share a mutable checkout) depends on absence being the default.
4. Per-step project provenance: when the mark names a directory inside a git work tree, the step record captures that repo's observed HEAD OID (the join is by pointer; project bytes never enter the workspace repo). No mark, or no repo — no field. Bundle/replay then render external project state honestly: resolving recorded OIDs is the reader's join, and a missing project repo is a named absence, never a guess.
5. Policy-blind throughout: lernie learns nothing of balls, targets, candidates, attempts, or delivery. The parameter is a directory; everything above it is the consumer's policy.

Downstream: yog consumes this via an exact published pin (its fire passes the claim-derived worktree; the goal-prose location preamble retires). Release and pin sequence live in yog VISION §4.10.