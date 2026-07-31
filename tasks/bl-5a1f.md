+++
title = "declared-is-not-callable holds only for the compactor: tool_step::run_tool_calls consults compactor::refusal alone, so every other role can run tools its grant omits but its inherited transcript declares"
created = 1785473896
updated = 1785473896
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["bug"]
+++
src/prompt/dispatch/tool_step.rs:79 gates execution via compactor::refusal(role, name), which returns None for every role != compactor. Request closure (§3.3) rightly declares every tool the inherited transcript names — but nothing then refuses execution for non-compactor roles, so a role can call any tool its ancestors ever used. Observed (fleet e2e /tmp/claude-1000/-home-mark-dev-lernie/d8b3c251-6d27-458f-a55c-166907ca93db/scratchpad/fleet-e2e/run-20260730-171212-2064167): a sensor role granted [slack_read, message] carried 'bash' in its request because its dispatcher had used bash; nothing would have refused a bash tool_use. This voids per-role grant boundaries (e.g. a read-only watcher on an outward surface) exactly when the dispatcher has used the guarded tool. ARCH §3.3 'declaring is not permitting' already promises the general rule. Fix: generalize the refusal seam — a tool_use naming a tool outside the role's providers.yaml tools: grant (plus role-injected tools, e.g. the compactor pair) is declined in-band with an is_error tool_result naming the role's grant; keep compactor behavior identical. Tests: an ungranted-but-declared call is refused for a non-compactor role; granted calls unaffected.