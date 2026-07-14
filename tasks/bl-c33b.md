+++
title = "epic: child step loop — dispatched children run to terminal and deposit (§2.5 live)"
created = 1783917121
updated = 1784004627
claimant = "Bottoming"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
tags = ["epic"]
+++
## Scope (reshaped per user, 2026-07-13)
Children never run: worker.rs stops at the dispatch commit. Land the user's dispatch shape — the parent/child relationship is essentially implicit, and dispatch is NOT a spawn:

1. Tool dispatch, inline and synchronous: fork the child branch off the given ref (default: commit where the dispatch landed), dispatch commit pins goal.md, soul.md, and the **workflow appendix** — a workflow fragment composed after the governing config's workflow.yaml, carrying one binding: terminal event → send the result message (epitaph/terminal ref/terminal response, §2.6) to the dispatching agent's address.
2. Deposit the dispatch message via lernie message — the front door; the deposit's own launch machinery starts the child nominally, like any agent.
3. Tool returns the child's id. Nothing supervises anything.

No child-specific loop, no worker path, and the step loop never branches on parent/child. Return totality becomes a property of the dispatch primitive (cannot fork without pinning the binding), not of loop code; the §8 died-sweep addresses its deposit by reading the same binding. Draft committed in this worktree (ARCH §2.5).

## Open pins (design, this ball)
- Appendix disk home: a dispatch-pinned file beside goal.md/soul.md (precedent: dispatch-pinned, model-visible) vs §2.2's control-out-of-worktree line. Needs settling.
- Return address: derivable from the id today; if the appendix names it explicitly, pick ONE authoritative home (SSOT). Explicit-in-appendix decouples report-target from descent (watchdog shapes) — lean explicit, confirm.
- goal.md vs dispatch message: whether the pinned goal duplicates the deposited text (as the root path does today) or distills it — settle against SSOT.
- Ripples to coordinate with bl-4684 (advance): lernie dispatch drops out of the §2.11 driver list (it becomes writer-shaped: fork+deposit+exit); child revival and first drive are advance's. worker.rs and the dispatch-as-driver path get deleted, not extended.

## Dependencies
Implementation lands against advance (bl-4684) as the launched driver; deposits and forks are buildable first, drive-by-advance last.