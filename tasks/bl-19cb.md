+++
title = "child_result.rs is 302 lines — over the 300-line code cap on main"
created = 1784525372
updated = 1784698006
claimant = "Prostheses-19cb"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Noticed by bl-9e2d (not caused by it — present on main at 70ac772, untouched by that ball's diff).

`src/prompt/dispatch/child_result.rs` is **302 lines**; the repo cap is 300 for code files (CLAUDE.md / AGENTS.md, enforced by .githooks/pre-commit).

It slipped through because the hook only measures **staged** files (`git diff --cached --name-only --diff-filter=ACMR`), so a file that crosses the cap in one commit and is untouched thereafter is never re-measured. Two things to fix:
1. Split child_result.rs back under 300.
2. Decide whether the hook should sweep the whole tree rather than only the staged set — a staged-only check cannot catch drift, and 'no code file over 300 lines' reads as a repo invariant, not a per-commit one. (Sweeping every commit costs a `find | wc -l` over src/, which is nothing next to the `make check` the hook already runs.)