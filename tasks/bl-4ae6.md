+++
title = "ARCHITECTURE §6 never spells out the max_depth boundary, so how far a child's own children may go is defined by an implementation guess"
created = 1785995944
updated = 1785995944
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
ARCH §6 lists `max_depth` as a budget axis but does not define what depth is measured from or which comparison exhausts it. `src/prompt/budget/mod.rs:31` says so outright:

> **`max_depth` and the root (flagged, ARCH §6).** §6 does not spell out the depth boundary. This module reads `max_depth` as the deepest allowed depth, so a driver is depth-exhausted iff `depth(branch) > max_depth`. The root is depth 0, so it is never depth-exhausted for any non-negative `max_depth`; a subagent `max_depth + 1` levels below the root exhausts on its first model call.

That reading may well be right, but it lives in the implementation rather than the spec, so the doc is not the single source of truth for it (PRINCIPLES) and nothing pins it — a later refactor could flip to `>=` and break no test that means to guard it.

This matters more than an ordinary spec gap because `max_depth` is the **only** limiter on a child dispatching children of its own. Per the bl-a4d5 operator ruling (2026-08-05) an agent is an agent: every agent may be messaged and may dispatch, and the sole sanctioned circumscription is an explicit prohibition. `max_depth` is that prohibition, so how deep the tree may go is a structural fact about the agent relation and belongs in ARCH §6 in words.

Deliverable: ARCH §6 states the boundary — what depth counts from, which comparison exhausts, and where the root sits — and a test pins the off-by-one at the boundary (a driver at exactly `max_depth` passes; at `max_depth + 1` it exhausts on its first model call). Delete the flagged note in `budget/mod.rs` once the spec carries it; the module should cite §6, not apologize for it.

Split out of bl-a4d5, which found it while auditing the prompt surfaces for claims that a child holds fewer powers than its parent.