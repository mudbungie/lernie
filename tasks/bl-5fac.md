+++
title = "gate: tests"
created = 1785125522
updated = 1785130104
claimant = "Halyard2"
parent = "bl-7318"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-7318"
on = "claim"
+++
PASS. `make check` green on the delivered tree (bl-7318, main 6a4a47d): fmt-check, `clippy --all-targets -D warnings`, tarpaulin **100.00% coverage, 5009/5009 lines, +0.00% change**, and the install contract (`cargo test --test install`). Run twice — once as the pre-commit hook on the work commit and once as the close gate — both EXIT=0.

New test landed: `prompt::inbox::tests::lock::release_frees_the_lease_while_a_subprocess_still_holds_its_fd`. It is a real proof, not a smoke test: it makes the fork window permanent and observable (close-on-exec cleared on the lease fd, the spawned child kept alive until the test closes its stdin) and asserts the lease is free the instant its guard drops. Verified to have teeth by reverting the `Drop` body — it fails without the fix and passes with it. No wall-clock verdict anywhere in it.

Determinism, the ball's stated bar: 30 x full `cargo test --lib` at default parallelism under 40 CPU spinners, loads 30-83, with two sibling `cargo tarpaulin --workspace` runs from other agents sharing the 16-core box. Both target tests (`parent_revival::a_child_final_response_revives_the_parent_which_delivers_and_steps`, `inbox::scan::tests::sweep::sweep_deposits_died_for_a_child_that_never_returned`) passed 30/30, as did `verifier_gate`, the third member of the same family found during reproduction. Pre-fix baseline on the same harness was 2 failures / 30.

One unrelated failure in that batch (`e2e::advance_cli::message_launches_a_detached_advance_chain_that_batons_through_tools`, a 120s evidence-poll bound outrun in an iteration where the whole suite took 121s against a normal 6-13s) is filed as bl-2bf0, not papered over here.