+++
title = "seam inversion: the router answers every tool invocation and the driver's local executor is deleted"
created = 1787977844
updated = 1788060214
claimant = "OrderExecutor"
priority = 3
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-2f58"
on = "claim"
+++
The downstream server's REMOTE §12 four-component ruling makes every execution cross the real wire to a separately installed tool host. The engine's half of that inversion, filed here and gated on the rename so it does not collide with that unlanded tree-wide commit.

**Generalize the seam.** `cmd::Fx::tool_injection` (ARCHITECTURE §3.3 *Host-injected tools*, `docs/DESIGN_TOOL_INJECTION.md`) today carries a router consulted *ahead of* the three-hop binary resolution, answering `Some` for the names it owns and `None` to fall through. Under the ruling there is nothing to fall through to: the router answers every tool invocation the agent makes, designated or not. The object and its two halves are unchanged — definitions spliced into the request and unioned into the grant gate, an injected name outranking an elected one — only its scope widens.

**Delete the driver's local executor.** Not a fallback, not a fast path for the co-located case, not a residual for names nobody routed. A driver that could still spawn a binary itself would be a second invocation pipeline with its own adjudication story, its own result envelope and its own containment claim, and which one an operator hit would depend on which names happened to be designated. Downstream states the destination as exactly one pipeline: adjudicate → mailbox → execute → capture.

**Refusal-in-band semantics are unchanged.** A routed invocation that cannot be delivered comes back as a non-zero result the model reacts to, never a hang and never a prefix change — the same contract the router already owes (its own deadline, a vanished endpoint rendered as a result, an eye on the cancel flag). Downstream, a server with no enrolled tool host therefore refuses every tool invocation in band; that is its ship-inert posture working, not an error state. The engine does not care where execution happens: it emits the invocation and waits on the capture.

**What this ball must price, not assume.** The exec binding passes `None` for the seam today, and the result envelope, the `is_error` mapping, the bounded projection and the per-invocation `input.json` / `output.json` record are landed by the executor rather than the router. Removing the local spawn must not remove those; decide explicitly where each lives afterwards and record it in the design doc. Whether the unlinked/exec binding keeps a local spawn at all is the priced decision — resolve it in `docs/DESIGN_TOOL_INJECTION.md`, do not drift into an arm.

Gates per this repo's guide: tests at 100% and green, docs updated (ARCHITECTURE §3.3 and `docs/DESIGN_TOOL_INJECTION.md` at minimum), alignment checked against ARCHITECTURE / PRINCIPLES / TAXONOMY.

Upstream authority: the downstream server's `docs/REMOTE.md` §5 (invocation path, as amended) and §12 (the split, front-door-only and ship-inert invariants). Filed from downstream yog bl-fe61, which carries the §5 amendment.