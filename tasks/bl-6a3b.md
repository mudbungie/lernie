+++
title = "epic: workflow actions — event→action bindings act at runtime (§6, v0.7)"
created = 1783917126
updated = 1784004696
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
workflow.yaml parses and validates (src/config/workflow.rs) and budgets enforce, but nothing consumes the event→action bindings at runtime. This epic lands the actions: dispatch(role), dispatch(role, with: ...), gate_return_on, deliver_result, compaction_merge, mark_abandoned, notify_ui — plus the per-step hooks (pre_step, post_step, on_tool_return). Success criterion is v0.7's: a verifier gating a worker's return runs end-to-end with zero code changes.

## Hook-home reconciliation (proposed 2026-07-13, awaiting user confirmation)
The contradiction (advance likes iteration granularity; the agent loop is the main operation) dissolves if **advance runs at step granularity and is the only driver**: one advance invocation = one step (drain → assemble → model call → tool loop → commit), then evaluate bindings, then exec the successor — the agent loop IS the advance chain, per §6's exec baton and §1 Regenerability ("crash recovery is the same lernie advance invocation that runs the chain in normal operation"). Per-step hooks are then advance-native (evaluated around the one step it runs); lifecycle events are the same evaluation when the step reached a terminal. One interpreter, one granularity, no split. lernie prompt and lernie dispatch stop being drivers — they become setup-plus-message writers (matches the bl-c33b dispatch reshape: fork + front door). Cost: one exec per step, trivial against a model call. Coordinate with bl-4684's design (advance) — this belongs in that living doc if confirmed.

## Must-pin items
- gate_return_on semantics: what holds a child's result delivery while a verifier runs — likely the parent's binding intercepts the result message before its delivery commit. Hardest action; design first.
- verifier_approve/reject derivation: from the verifier's result message (epitaph + terminal response), on disk, no sidecar.
- Actions stay a closed set; new actions are code, bindings are config.
- The bl-c33b workflow appendix composes after the config workflow at read time — the appendix's terminal→result-message binding is just another binding this interpreter executes.

## Blockers
Claim-blocked on bl-4684 (advance is the interpreter) and bl-c33b (a verifier variant needs children that run).