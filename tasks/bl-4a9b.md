+++
title = "Fix v0.6 stragglers: prompt_retry.rs git-env leak + per_repo_providers.rs stale providers.yaml strings"
created = 1783490289
updated = 1783490318
claimant = "Sandcastle-4a9b"
priority = 5
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Two defects found during the bl-660b docs gate (main 30e68f2), reported not fixed there.

1. PRIMARY (correctness/robustness). tests/prompt_retry.rs (~lines 193-199, retryable_529_then_clean_writes_two_segments_and_completes): the 'git -C <primary> branch --list --no-merged main' subprocess does NOT scrub the ambient git env. Under a GIT_DIR/GIT_INDEX_FILE-exporting environment (i.e. whenever git runs the pre-commit hook, which runs tarpaulin, which runs this test), the inherited GIT_DIR overrides the -C target and redirects the query off the fixture repo -> stdout non-empty -> assertion fails. Deterministically reproducible: fails under the pre-commit hook, passes standalone. This is the SAME fork/exec env-inheritance hazard already mitigated in production by git_tree::cmd::INHERITED_GIT_ENV and patched into tests/pluggability.rs (see commit d53ca19). FIX: apply the same INHERITED_GIT_ENV scrub to this test's git subprocess. Why it matters: bl-660b had to --no-verify its local commit because of this; bl close's gate only passes because its delivery env doesn't leak GIT_DIR. Any future hook-run of the suite keeps hitting it.

2. SECONDARY (stale strings). src/config/per_repo_providers.rs lines 7, 53, 74 reference the global '<harness-root>/providers.yaml' but v0.6 (bl-56ee) renamed the global file to models.yaml. Line 74 is a user-facing LoadError::Invalid message pointing users to a nonexistent file. Straggler from the rename. FIX: update the three strings to models.yaml (or the correct per-repo vs global distinction).

Gates: tests (100% coverage holds; add/adjust so the retry test passes under a GIT_DIR-leaking env), and confirm cargo test --workspace green. <=300 line files, no banned terms.