+++
title = "Lost wakeup in the 2.11 deposit-probe-launch protocol: a deposit racing a driver's last inbox read is stranded forever"
created = 1785130626
updated = 1785133272
claimant = "Gudgeon"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
STATE AT HANDOFF (Tumult, 2026-07-27, after agent Gudgeon died silently mid-close — recorded per user instruction; work NOT lost):

DONE by Gudgeon:
- Full implementation committed: work/bl-9c8f branch, single commit bc86a71 "Close the §2.11 lost wakeup: the release rule, the deposit rule's dual" — 7 files, +339/-26, including new test files src/prompt/dispatch/driver/release_tests.rs and src/prompt/tests/advance_release.rs.
- Per Gudgeon's last report: ARCH §2.11 amended (deposit rule and its dual side by side), alignment self-checked clean (seen-set is one invocation's own derived read, no sidecar state; warrant decided under the lock by the launched driver; "release rule" defined in the introducing document).
- The pre-commit gate RAN TO COMPLETION as an orphaned setsid process after the agent died and PASSED: check.exit = 0 (check.log retained in the worktree, ~145KB).

ORPHANED WORKTREE: /home/u/.local/state/balls/plugins/bl-delivery/home/u/dev/lernie/bl-9c8f on branch work/bl-9c8f at bc86a71, clean apart from untracked check.log/check.exit. A future bl claim re-attaches it.

REMAINING for whoever picks this up:
1. git merge main in the worktree (main has advanced: bl-2bf0, other-session UX balls), re-run the gate if the merge is non-trivial.
2. The 30-consecutive-run load proof of the advance_cli baton test (~40 spinners) that Gudgeon queued but never ran — the residual ~1-in-8 flake bl-2bf0 documented is the acceptance signal.
3. rm check.log check.exit, bl close, sweep gate children bl-193c (docs) / bl-e716 (tests) / bl-ec93 (alignment).
4. Close-overflow hazard if any: reset --hard main + cherry-pick bc86a71, never a saved patch.

Original defect record follows in prior body revisions (bl show history): lost wakeup in the §2.11 deposit→probe→launch protocol, discovered with 100% repro by Cotter in bl-2bf0.