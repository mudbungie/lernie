+++
title = "gate: docs"
created = 1785124069
updated = 1785125303
claimant = "Cringle"
parent = "bl-d32a"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-d32a"
on = "claim"
+++
PASS, with one correction made rather than merely noted.

README: new '### Auto-push hook' section under Contributing — what the hook does, why `reference-transaction` and not `post-commit` (bl delivers by plumbing, so no commit or merge hook fires on a landing), exactly what it ignores (side branches, `refs/remotes/*`, no-op rewrites, a deletion of main), why it can never block or hang a landing (git aborts a ref transaction on a non-zero hook in the `prepared` state, hence every path exits 0 and hence no `set -e`), the git >= 2.28 floor, and what `tests/hooks.rs` proves. Cross-linked from the Workflow section ('every landing on `main` is pushed to `origin` automatically') and from Contributor setup, where `make install-hooks` now says it arms both hooks, not 'the hook'.

CORRECTION FOUND AND FIXED: both the hook comment and the README justified the mechanism with a census of main's history — '199 of 199 commits ... zero are merge commits'. main took a merge commit (`0a6a36a`, origin/main folded in) the same day, so the claim was false within hours of being written. Replaced with the durable fact — bl delivers by plumbing, so no commit or merge hook fires — which is what the argument actually rests on. Landed as `e011397`.

Checked and correctly left alone: `.github/workflows/ci.yml` and `release-plz.toml` (the remote side was already complete); ARCHITECTURE/PRINCIPLES/TAXONOMY (covered by the alignment gate, bl-ea16); `Cargo.toml`'s `exclude`, which already carries `/.githooks` and `/tests`, so neither new file reaches the published crate.

Gate on the correction: `make check` green — 100.00% coverage, 4678/4678, +0.00% change; install contract 1 passed.