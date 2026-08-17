+++
title = "the shipped whole-tree spend ceilings bind far too early on ordinary chats: ship the template with budgets off by default"
created = 1786937007
updated = 1786937007
priority = 1
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Operator ruling, 2026-08-16: the shipped spend ceilings bind far too early on
ordinary chats. **Disable them by default — the whole block.**

## What ships today

`template/workflow.yaml`:

    # Whole-tree spend limits (ARCH §6 "Budgets (v0.7)"). One frozen ceiling
    # for the whole agent tree, not a per-agent allowance: every driver in
    # the tree — root or subagent — checks the tree's total against these
    # same numbers, and a dispatch inherits no fresh budget. Checked at
    # every model-call boundary before the adapter is invoked; spend, wall,
    # and depth are derived from disk each check — no stored counter. Omit a
    # limit (or the whole block) to leave that axis unbounded.
    budgets:
      max_total_tokens: 2000000
      max_wall_seconds: 3600
      max_depth: 4

**Whole-tree, not per-agent, is why these bite so early.** A root that
dispatches subagents spends one shared allowance, so the ceiling a chat actually
meets is far below the number as written. `max_wall_seconds: 3600` is the
sharpest edge: one hour of accumulated wall across the whole tree ends a
conversation that is working correctly.

## The change

Delete the `budgets:` block from the shipped template. The block's own comment
already documents this as the supported way to be unbounded — "Omit a limit (or
the whole block) to leave that axis unbounded" — so this is **config deletion,
not a code path**, which is exactly the severability the house rule asks for:
removing a default should delete config, not edit code. Verify that claim in
`src/` before you rely on it: an absent block must read as unbounded on all
three axes with no branch of its own, and if any axis instead defaults to a
compiled-in number when the key is missing, THAT is the defect and the ball is
about fixing it.

Do not replace the numbers with bigger numbers. The ruling is off by default,
not generous by default; an operator who wants a ceiling adds the block back.

## `max_depth` — flagged, and going with the rest unless overruled

`max_depth: 4` is not a spend ceiling in the same sense: it is the guard against
unbounded recursive dispatch, where an agent that dispatches an agent that
dispatches an agent has no natural floor. Removing it is what "disable it
entirely" says and it is what this ball asks for, but it is the one axis whose
absence can produce a runaway rather than merely an expensive chat. Implement
the ruling as written; if the operator revises it to "keep depth", that is a
one-line amendment to this ball and not a redesign.

## The half a template change does NOT fix

A workspace freezes its own copy of `workflow.yaml` at creation. Editing the
template governs **new** workspaces only — every conversation already on disk
keeps the ceilings it was born with. State this plainly wherever the change is
announced, and check whether any supported path re-reads or refreshes a frozen
config; if none exists, say so rather than implying the fix is retroactive. Do
not build a migration under this ball — file it if it is wanted.

## Also check, do not widen

`template/manifest.yaml` carries `budget_tokens: 150000` and `budget_tokens:
50000`. Establish whether those are the same ceiling family or an unrelated
per-role thinking allowance, and report which. Change them only if they are the
same thing the ruling names.