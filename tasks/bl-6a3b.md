+++
title = "epic: workflow actions — event→action bindings act at runtime (§6, v0.7)"
created = 1783917126
updated = 1783917126
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["epic"]

[[blockers]]
id = "bl-4684"
on = "claim"

[[blockers]]
id = "bl-c33b"
on = "claim"
+++
## Scope
workflow.yaml parses and validates (src/config/workflow.rs) and budgets enforce, but nothing consumes the event→action bindings at runtime. lernie advance (bl-4684) is the interpreter home for workflow-event boundaries; this epic lands the actions behind it: dispatch(role), dispatch(role, with: ...), gate_return_on, deliver_result, compaction_merge, mark_abandoned, notify_ui — plus the per-step hooks (pre_step, post_step, on_tool_return). Success criterion is v0.7's: at least one non-baseline variant (a verifier gating a worker's return) runs end-to-end with zero code changes.

## Must-pin items
- Frequency split, needs ARCH pinning: per-step hooks fire at step frequency and belong to the *executor* (which already reads the governing config); agent-lifecycle events (worker_return, verifier_approve/reject, compactor_return, branch_stopped) belong to *advance* at workflow-event boundaries. §6 currently reads as if advance evaluates everything; the split must be written down or the executor grows a second interpreter.
- gate_return_on semantics: what holds a child's result delivery while a verifier runs — likely the parent's workflow binding intercepts the result message before delivery-commit. Needs design; this is the hardest action.
- Action strings are a closed set (workflow.rs already validates); new actions are code, bindings are config — keep it that way.
- verifier_approve/reject event derivation: from the verifier's result message (epitaph + terminal response), on disk, no sidecar.

## Blockers
Claim-blocked on bl-4684 (advance is the interpreter) and the child step loop epic (a verifier variant needs children that run).