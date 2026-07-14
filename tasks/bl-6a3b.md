+++
title = "epic: workflow actions — event→action bindings act at runtime (§6, v0.7)"
created = 1783917126
updated = 1784010044
claimant = "Reappear-6a3b"
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

## Hook-home reconciliation (proposed 2026-07-13, user leaning yes — "matches one path, through the front door")
Advance runs at STEP granularity and is the only driver: one advance invocation = one step (adopt-or-acquire lock → drain inbox → assemble → model call → tool loop → commit → evaluate bindings), then exec the successor. Per-step hooks are advance-native; lifecycle events are the same evaluation when the step reached a terminal. prompt/dispatch stop being drivers — setup-plus-message writers. Coordinate with bl-4684 (advance design, Companions-4684).

## No loop component — the kernel is the loop (2026-07-13 Q&A)
There is no caller invoking .advance() repeatedly and no wrapper while-loop. exec replaces the process image in place: same pid, same inherited lock fd (bl-4684's CLOEXEC/LERNIE_LOCK_FD pin — the flock rides the open file description, so the lease is continuous with zero re-acquisition gap). The chain IS the loop; termination = a terminal binding evaluates → exit protocol → exit without exec. The drain sits at the TOP of each iteration (end-of-step-N and start-of-step-N+1 are the same instant by exec); the graceful-exit crack is the §2.11 exit-launch; a quiescent agent is NO process, revived by the next deposit. A wrapper loop was considered and rejected: it is a resident component holding "am I still looping" in memory (second home for a disk-derivable fact, violates §6 position-derivability), has a separate crash fate from its child, and muddles lock ownership (holds it = it is the executor; re-acquires per call = lease gap per step). §6 verbatim: "the currently-executing lernie advance subprocess is the interpreter while it runs — a baton passed forward by exec, not a daemon." Cost: one exec per step, milliseconds vs a model call.

## Must-pin items
- gate_return_on semantics: what holds a child's result delivery while a verifier runs — likely the parent's binding intercepts the result message before its delivery commit. Hardest action; design first.
- verifier_approve/reject derivation: from the verifier's result message (epitaph + terminal response), on disk, no sidecar.
- Actions stay a closed set; new actions are code, bindings are config.
- The bl-c33b workflow appendix composes after the config workflow at read time — its terminal→result-message binding is just another binding this interpreter executes.

## Blockers
Claim-blocked on bl-4684 (advance is the interpreter) and bl-c33b (a verifier variant needs children that run).