+++
title = "stop during tool execution: wire SIGTERM to run_tool_calls stop_flag for the clean stopped-deposit exit"
created = 1783916329
updated = 1783916902
claimant = "Companions-f2a8"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Problem (found during bl-aafc)
`run_tool_calls` uses a local `stop_flag` disconnected from the SIGTERM handler, and a group-killed tool returns `Err(KilledBySignal)` that propagates — so a `lernie stop` landing during a tool-execution window makes the harness exit non-zero, not the clean `stopped`-deposit exit ARCH §2.9 step 3 describes for the model-call window. Discovery (bl-aafc) now finds the pid in that window via the inbox lock fd; exit behavior there is still the crash shape.

## Fix direction
Wire the SIGTERM handler to the tool-execution path so a stop during a tool window follows the same terminal sequence as the model-call window: kill the tool subprocesses (pgid, §2.9 steps 1-2), deposit the `stopped` epitaph, exit clean. See bl-aafc's test `stop_lands_during_tool_execution_via_inbox_lock_fd` (asserts termination only, not clean exit) — tighten it to assert the stopped-deposit once this lands.

## Deliverables
- SIGTERM-connected stop flag across tool execution; KilledBySignal in the stop path becomes the stopped exit, not an error propagation.
- ARCH §2.9 wording if it distinguishes windows.
- Test: stop during tool window produces the stopped epitaph deposit and clean exit.