+++
title = "gate: alignment"
created = 1785124418
updated = 1785124445
claimant = "Ratchet"
parent = "bl-9300"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-9300"
on = "claim"
+++
PASS with one noted residual.

Checked the shipped state of the ETXTBSY retry envelope against docs/ARCHITECTURE.md, docs/PRINCIPLES.md and docs/TAXONOMY.md.

Coherent:
- ARCH §3.3 governs tool spawn; the retry envelope is an implementation detail of that spawn and the injected budget changes no §3.3 contract. Production ETXTBSY_RETRY_BUDGET stays at 200ms, so shipped behaviour is untouched — the test-only override is the whole change.
- PRINCIPLES "Testability via configuration" — the budget became a per-executor injected parameter (SpawnTool::with_etxtbsy_budget) rather than a shared wall-clock constant the test races. That is the principle applied, not bent.
- PRINCIPLES "Single source of truth" — the shipped default still has exactly one home (subprocess::ETXTBSY_RETRY_BUDGET); SpawnTool::new reads it, and only a caller that explicitly overrides carries its own.
- TAXONOMY.md — no new term of art introduced anywhere in this work.

Residual, recorded not fixed: README.md states in the coverage-determinism section "A retry budget is a count of attempts, never a wall-clock deadline." The production envelope is still a wall-clock deadline (Instant::now() < deadline in spawn_with_etxtbsy_retry). The line covering its retry arm is covered by spawn_surfaces_etxtbsy_after_budget_exhausted, which is safe by a different argument than the README rule prescribes: the holder keeps the fd open permanently so every attempt is guaranteed to see ETXTBSY, and the give-up arm can only pre-empt the retry arm if a whole 1s injected budget elapses between computing the deadline and the first failed spawn — implausible, but a probability argument rather than a structural one. Converting the envelope from a Duration to an attempt count would satisfy the README rule structurally. Not in scope for this ball (which delivered no source change) and not worth churning a third time on this file without a decision; raised for the architect.