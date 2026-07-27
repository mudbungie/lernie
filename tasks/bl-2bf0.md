+++
title = "e2e::advance_cli baton test outruns its 120s evidence-poll bound when the box runs several full suites at once"
created = 1785130071
updated = 1785130161
claimant = "Cotter"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Observed by Halyard2 during bl-7318's post-fix determinism run (2026-07-26), run 25 of 30: `e2e::advance_cli::message_launches_a_detached_advance_chain_that_batons_through_tools` panicked with `timed out waiting for "/tmp/.tmpqCSG6p/conv/agents/<id>/messages/003-user.md"` (src/e2e/advance_cli.rs:156). That iteration's whole `cargo test --lib` took 120.97s against a normal 6-13s: 40 synthetic spinners plus TWO sibling `cargo tarpaulin --workspace` runs from other agents' gates, load ~58-83 on 16 cores. The other 29 iterations passed; bl-7318's own two target tests passed 30/30 in the same batch.

bl-6987's census classed this poll SAFE with the reasoning 'evidence polls: the pass path is satisfied by observable state however slow it arrives; the bound exists only to convert a hang into a failure with diagnostics. Bounds are 10-100x the bounded work and were exercised at load 65-85 in this ball's verification.' The reasoning is right in kind and the margin was simply outrun — the box was running roughly three full test suites at once, ~20x slower than the census conditions.

USER DIRECTIVE (inherited from bl-6987/bl-7318): fix root causes; no retries, no widened tolerances, no ignores, no weakened assertions. So bumping 120s is NOT the fix. The question to answer is whether the detached advance chain's progress can be waited on by observable state with no wall-clock bound at all (a chain that has provably exited vs one still working is a derivable distinction — the executor lock's liveness probe already answers 'is anyone driving this branch', ARCH 2.11), leaving the bound as a pure hang-diagnostic that the pass path never approaches. Same question applies to the sibling e2e polls bl-6987 listed (stop_common poll_for_path/branch, replay_drive, stop_children PATIENCE).

Bar: 30 consecutive passes under synthetic load (~40 spinners, load 30+).