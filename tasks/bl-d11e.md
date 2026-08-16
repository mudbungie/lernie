+++
title = "the changelog guard resolves the last release tag via git describe, which committer-date skew misorders: it answered v0.0.1 (113 commits) while v0.0.8 sat 3 commits from main, false-firing on an ancient delivery and blocking every close in the checkout"
created = 1786843603
updated = 1786844835
claimant = "Dills"
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## The defect

`tests/changelog_completeness.rs` resolves the range base with

    git describe --tags --abbrev=0 --match v* refs/heads/main

`git describe` walks commits in committer-date order, and this history carries date skew (a parent whose committer date postdates its child by hours — merge 48be6fe, 2026-08-13T21:16Z, whose parent d16cba9 reads 2026-08-14T04:15Z). Under that skew describe returned `v0.0.1-113-gd8ac11f` even though `git rev-list --count v0.0.8..main` is 3; its own `--debug` listed v0.0.2..v0.0.8 all at depth 114 of 115 traversed. The guard then judged the range v0.0.1..main and failed on bl-8f7c — a delivery three releases old — refusing an unrelated close. (Unblocked 2026-08-15 by restoring that bullet under its own 0.0.2 section, in bl-15f0's delivery; the resolution itself is still skew-fragile.)

## Shape of the fix

Do not ask describe. Enumerate `v*` tags, keep those with `git merge-base --is-ancestor <tag> refs/heads/main`, and take the one minimizing `git rev-list --count <tag>..refs/heads/main` — the nearest reachable release by the counting the guard itself uses. Same skip behavior when no tag qualifies. Pin it with a regression exercising a skewed history.