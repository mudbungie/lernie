+++
title = "Auto-push main to origin on every merge (post-commit hook)"
created = 1785124069
updated = 1785124343
claimant = "Capstan"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
SHIPPED as `.githooks/reference-transaction`, not `post-commit` — the decided mechanism was inert on this repo and the correction is the deliverable.

MEASURED: `bl close` does not run `git commit`. bl-delivery squashes with `git commit-tree` and advances the branch with `git update-ref refs/heads/main` (verified by strings on the plugin binary AND end-to-end: a scratch bl repo with probe hooks for post-commit / post-merge / pre-commit / reference-transaction saw ONLY pre-commit — fired in the work worktree, on branch work/<id> — and then a bare `refs/heads/main` ref transaction). So a post-commit hook gated on 'HEAD is main' would have fired on zero of the 199 commits on main, all of which are squash deliveries; main carried zero merge commits at the time of writing.

MECHANISM: git's `reference-transaction` hook is the one event every landing path shares. It acts only on state `committed`, only on `refs/heads/main`, only when the value actually changes and is not all-zeros, and only when an `origin` remote exists; then `timeout 30 git push --quiet origin main`. A failure prints one warning line on stderr. Side branches, `refs/remotes/*` (including the ones its own push writes — hence no recursion), `git pack-refs` no-ops, and a deletion of main all fall through. No `set -e`: git ABORTS a ref transaction when this hook exits non-zero in the `prepared` state, so every path exits 0 — the hook cannot block or fail a landing. Needs git >= 2.28 (2.53 here).

Constraints from the decided design all hold, with 'HEAD is main' generalised to 'the transaction moves refs/heads/main' (HEAD is never main during a plumbing delivery, which is exactly why the original phrasing could not work).

PROOF: `tests/hooks.rs` — 6 tests against a LOCAL BARE REPOSITORY as origin, never the real remote: commit on main pushes; `commit-tree`+`update-ref` delivery pushes (the path that matters); `git merge --no-ff` pushes; a side-branch commit pushes nothing and creates no remote branch; an unreachable origin warns without failing the commit; no origin at all is silent. Mutation-checked: neutering the hook to `exit 0` fails 5 of the 6. Also proven end-to-end through a real `bl close` in a scratch bl repo with a bare origin — the squash reached the remote.

DOCS: README 'Auto-push hook' section (mechanism, why not post-commit, the never-block argument, the git floor), plus the Workflow and Contributor-setup sections. Untouched as instructed: ci.yml, release-plz.yml.

GATE: `make check` green — 100.00% coverage, 4678/4678 lines, +0.00% change; all tests pass; no source file over 300 lines (tests/hooks.rs 255, hook 33 and `.githooks/` is cap-exempt).