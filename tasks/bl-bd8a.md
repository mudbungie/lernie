+++
title = "workflow actions v0.7: remaining executors + verifier-gate end-to-end [follows bl-6a3b]"
created = 1784010384
updated = 1784010384
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["epic"]

[[blockers]]
id = "bl-ac5a"
on = "close"
+++
## Scope (follows bl-6a3b)
bl-6a3b landed the §6 binding interpreter (advance-native), reconciled the closed action/event set to the current post-merge-back names (gate_return_on, deliver_result, compaction_merge, compactor_return; spawn_root_agent), threaded the parsed workflow bindings through resolve::WorkerConfig, and wired the FIRST runtime lifecycle binding: branch_stopped -> [mark_abandoned, notify_ui] as git-native ref marks (refs/lernie/abandoned/*, refs/lernie/notify/*), executed config-only by lernie advance at the stopped terminal. Design for the whole feature is in ARCHITECTURE.md §6 'The binding interpreter (v0.7)'.

## Remaining (the v0.7 success criterion: verifier gating a worker's return, config-only, end-to-end)
1. Per-step hook firing points: pre_step / post_step / on_tool_return evaluated inside hop::step (advance-native — before the model call, after it returns, after each tool resolves).
2. worker_return -> dispatch(verifier) + gate_return_on(verifier.approve): the delivery-hold. A drained worker result is NOT delivered; a verifier is dispatched off the worker's terminal ref; the held state is disk-derived (worker result undelivered in inbox + verifier dispatched + not yet approved). Role of a delivered child result is derived from the child's dispatch commit subject (dispatch: <role> [<branch>]) — no sidecar. Design pinned in §6.
3. verifier_approve / verifier_reject derivation from the verifier's result message (epitaph == final-response + terminal-response verdict token). approve -> deliver_result (the held worker result drains: transfer + delivery commit). reject -> dispatch(worker, with: verifier.feedback).
4. deliver_result and compaction_merge executors. compaction_merge coordinates with the real-compaction work (bl-9dbd) — its compactor-return merge is the same one merge; do not duplicate.
5. Collapse lifecycle-binding evaluation onto the root run_exchange path too (today only advance evaluates branch_stopped), per the §6 prompt->advance collapse.

## Success criterion (unchanged from v0.7)
A verifier gating a worker's return runs end-to-end with ZERO code changes (config only), demonstrated by a test using the stub-adapter harness.