+++
title = "gate: docs"
created = 1785125509
updated = 1785129526
claimant = "Oakum"
parent = "bl-1c94"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-1c94"
on = "claim"
+++
Gate: docs for bl-1c94 — **PASS**; docs updated in the same commit 6d4412b.

- `docs/ARCHITECTURE.md` §2.6 *Merge is reserved for compaction*: the
  merge lands "only for a **`final-response`** return: the epitaph is
  the proof the pass completed, so any other value (`died`, `stopped`,
  `budget-exhausted`) lands no merge, the result delivering like an
  ordinary child's instead (§2.7)."
- §2.6 shipped-state note (bl-9dbd) now names the enforcement point:
  "enforced at the delivered-result interpreter (bl-1c94):
  `compaction_merge` is epitaph-gated in `child_result.rs`, and a
  non-`final-response` compactor return is delivered like an ordinary
  child's result instead".
- §6 closed-action-set paragraph and the bl-2d5a shipped-state note
  carry the same rule ("epitaph-gated wherever it is bound").
- `README.md` "Dispatching subagents directly" already promised the
  rule; it now records that the harness enforces it.

No stale claim of unconditional merging remains (`grep` over README and
ARCH for `compaction_merge` / `compaction merge`).
