+++
title = "Budgets: per-conversation spend limits (v0.7 workflow scope)"
created = 1783464230
updated = 1783490465
claimant = "Sandcastle-9e9b"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-56ee"
on = "claim"
+++
From ~/dev/harness spec §8, folded per ARCHITECTURE v0.4 §6/§12. workflow.yaml declares budgets: max_total_tokens, max_wall_seconds, max_depth. Enforced at every model-call boundary before invoking the adapter; spend accumulated from Usage events in response.json across the conversation tree (derived at check time — no sidecar counters, PRINCIPLES single-source-of-truth). A dispatch hands the child min(parent remaining, child declared). Exhaustion = harness-issued stop + refs/lernie/budget-exhausted/<branch> ref (mirrors the conflicted-ref pattern, §2.6 step 6); await(handle) surfaces {status: budget_exhausted}. Exhaustion is an ordinary terminal state, never a special transcript shape.