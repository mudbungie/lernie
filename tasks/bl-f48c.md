+++
title = "Collapse §6 budgets to one live whole-tree check; delete clamped inheritance"
created = 1783645920
updated = 1783645924
claimant = "Chandlers"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-4ca9"
on = "close"

[[blockers]]
id = "bl-c406"
on = "close"

[[blockers]]
id = "bl-32ca"
on = "close"
+++
## Finding
ARCH §6 clamped inheritance (remaining + clamp + a dispatch-time snapshot handed to the child) is redundant machinery. It was designed as if a subagent could not see its siblings' live spend. But steps/ is ONE shared tree at the conv-repo root, outside every worktree, NOT in git, namespaced by conv-id, never merged: §2.2 'steps/ ... shared across the whole conversation tree and namespaced by conversation id'; §2.3 'every conversation in the tree (root + every subagent) writes into a single shared steps/ tree'; §2.6 'they share the conv-repo's steps/ tree from the moment they're written.' So any driver can derive the WHOLE tree's live spend with no merge, no snapshot, no inheritance. The double-counting caveat in budget/mod.rs ('cannot be re-derived once the child starts spending') is self-inflicted: it only appears because the design computes parent-minus-child instead of reading the whole-tree total once.

## The reframe (what §6 should say)
Three separable things, none requiring up-the-stack attribution:
- depth: positional, hyphens/2 off the branch name (unchanged).
- global ceiling: one live derive(root_of(branch)) = whole tree, vs the single frozen limit, checked by whoever is driving (root OR subagent). derive already sums branch + descent, so deriving against the root id = the whole tree.
- optional local subtree cap: a future --token-cap knob checked against a subtree's own spend. NOT built here (new flags are a smell); noted as future.

Clamped inheritance does not even do its stated job: with one frozen workflow.yaml per tree, child_declared == parent_declared, so the clamp is always a no-op; and under concurrent dispatch each child snapshots the same remaining and can collectively overshoot. The live whole-tree derive is the only thing that actually holds the ceiling.

## Edits
DOC docs/ARCHITECTURE.md §6: replace the sentence 'A dispatch hands the child the minimum of the parent's remaining budget and the child's own declaration (per axis; tokens/wall deplete, depth being a shared absolute ceiling).' with the derived whole-tree model above. Note --token-cap local cap as future-not-built.

CODE src/prompt/budget/mod.rs: delete remaining and clamp (+ their unit tests in tests.rs; ball bl-1067 confirmed zero non-test consumers). Rewrite the 'Clamped inheritance' module doc-comment; delete the 'cannot be re-derived once the child starts spending' caveat (moot). Generalize check with a one-line root_of(branch) helper: derive tokens/wall over root_of(branch) (whole tree), depth over branch. This is ONE function that already does the right thing for the root (root_of(root_id)==root_id) and now also any subagent driver — the small honest remainder from the design discussion ('a non-root driver must also run the global check'), landing as a helper not a mechanism.

## Disposition of bl-1067
Its premise (wire the clamped hand-off into the subagent step loop) evaporates — there is no hand-off to wire; the subagent step loop just calls the generalized check. Close bl-1067 pointing here.

## Confidence
Rests entirely on: a subagent reads the whole tree's live spend via shared steps/. §2.2/§2.3/§2.6 confirm unambiguously. A confirmation test (two conv-id step dirs; assert derive::spend(root_id) sums both) is part of the tests gate.