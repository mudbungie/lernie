+++
title = "gate: alignment"
created = 1785125509
updated = 1785129526
claimant = "Oakum"
parent = "bl-1c94"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-1c94"
on = "claim"
+++
Gate: alignment for bl-1c94 — **PASS** against `docs/ARCHITECTURE.md`,
`docs/PRINCIPLES.md`, `docs/TAXONOMY.md`.

- **TAXONOMY** (*epitaph*, l.285): "as a total field, code branches on
  its *value*". `merge_qualifies(cr)` branches on `cr.epitaph ==
  Epitaph::FinalResponse` — value, not message shape; the single
  delivery path is preserved (a non-qualifying return falls through to
  the ordinary `deliver_result`). No new term coined.
- **PRINCIPLES / Structure over discipline**: the gate sits on the
  *action* (`Action::CompactionMerge`), not on the default binding, so
  no `workflow.yaml` can configure an unfinished compactor into a
  merge. Pinned by
  `a_stopped_compactor_return_lands_no_merge_under_an_explicit_binding`.
- **A special case is usually a missing reframe**: no new action, no new
  event, no new config key — one predicate on an existing field, and
  the non-merge case reuses the general child-return path.
- **ARCH §2.7** ("surfaced for user review like any other child
  failure") is satisfied literally: the epitaph lands in the parent's
  transcript and the branch continues uncompacted.
