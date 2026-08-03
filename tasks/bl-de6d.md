+++
title = "connection points in the tool-call path: a gate seam, and controls shipped as knobs"
created = 1785650728
updated = 1785724357
claimant = "Cleat"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Operator ruling (2026-08-01, codex-comparison follow-up), verbatim: "big gap. there is the part that says, appropriately, that this is all workflow tooling. you could build a workflow with controls. but, it doesn't ship out of the box, and it would be worth it to do so, even as knobs. and, I don't think I have the hooks in place to put it in the way of tool calls, which is a big leverage point around what should get reviewed. Needs more connection points."

RE-SCOPED same day by operator, verbatim: "the answer on controls at this point is that we should wire the capabilities, but not necessarily the implementations."

## The deliverable: the seam only
A connection point IN the tool execution path — before a tool_use block executes (the loop at src/prompt/dispatch/tool_step.rs:76 drives blocks serially through the executor; verify current shape) — where a configured control can pass, refuse, or hold a call for review. Today the workflow action set is closed (dispatch, gate_return_on, deliver_result, compaction_merge, mark_abandoned, notify_ui) and none of them can sit in front of a tool call; this ball adds that class of connection point. Design open: a workflow-bound predicate per role/tool, a control binary invoked like a tool, or a gate ball — decide inside, but the seam must be config-wired, not hardcoded policy.

Testability bounds the minimum: the seam ships with a test-fixture control proving pass/refuse/hold end-to-end (if it can't be tested it mustn't be built). That fixture is a test asset, not a shipped control.

## Explicitly OUT of scope, by the re-scope ruling
Shipped production controls (the "out of the box, even as knobs" half of the original ruling). When one is wanted it files as its own ball and lands as severable config — deleting it deletes config, not code. Nothing in the seam's design may assume a particular control exists.

## Codex reference (openai/codex @ 2b5bdcf)
Their leverage points: run_pre_tool_use_hooks / run_post_tool_use_hooks / permission-request hooks (codex-rs/core/src/hook_runtime.rs) and the guardian — an LLM control that adjudicates approvals with a compact transcript, failing closed (codex-rs/core/src/guardian/). The guardian shape maps cleanly onto lernie: a verifier-role dispatch is already the house pattern; the missing piece is only the seam that puts it in the path.