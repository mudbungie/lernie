+++
title = "gate: tests"
created = 1785125509
updated = 1785129529
claimant = "Oakum"
parent = "bl-ee80"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-ee80"
on = "claim"
+++
Gate: tests for bl-ee80 (a failed model call is a named silent death) —
**PASS**.

- CI on the landing commit **6d4412b** is green: run 30220130316 (`CI`,
  push, main) — fmt-check, clippy `-D warnings`, tarpaulin
  `--fail-under 100`, install contract.
- Re-run locally by Oakum in a worktree at 6d4412b, all exit 0:
  `cargo test --workspace scan` (35 passed, 0 failed),
  `cargo test --workspace message` (53 passed, 0 failed),
  `latest_step_outcome` and `died` filters likewise clean. The tests
  this ball landed all pass by name:
  `prompt::inbox::scan::tests::sweep::a_root_with_a_failed_model_call_is_a_named_silent_death`,
  `...::sweep::a_failed_latest_response_is_a_death`,
  `e2e::scan_cli::scan_names_a_root_whose_model_call_failed`,
  `cmd::tests::verbs_more::message_advises_on_a_quiescent_branch_whose_latest_call_failed`.
- The pre-existing sweep suite was migrated to the `Vec<String>` report
  shape and still passes (idempotent double scan, driven child never
  swept, delivered/undelivered child cases).
