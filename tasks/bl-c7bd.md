+++
title = "gate: docs"
created = 1785124820
updated = 1785129535
claimant = "Parrel"
parent = "bl-f021"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-f021"
on = "claim"
+++
PASS. Docs describe the shipped state, verified against the landed diff (main 440da96).

- **ARCH §2.7** — new paragraph **"Declared is not callable"**: why a compactor's request names the inherited transcript's tools (a provider validates the request as a whole and refuses a history naming an undeclared tool), why it still cannot call them, and why the alternative — rewriting/textualizing the inherited \`tool_use\` blocks — is rejected (§2.3 immutability, §3.3 transcript-backed framing).
- **ARCH §3.3** — new rule **"The array is closed over the history it ships"**, stated as the general composer rule (election reads one way, closure the opposite: a name the history references is never dropped), incl. the bare \`{"type":"object"}\` stand-in for a model-invented name; plus a **Shipped-state note (bl-f021)** naming the real seam \`compose(worktree, role, declared, history)\`, both step drivers, \`compactor::refusal\` in \`tool_step.rs\`, the pre-fix symptom, and the two test files.
- **ARCH §4.3** — the paired-toolset paragraph extended: declared is wider than the pair, callable is exactly the pair, "the safety property is the second fact, not the first".
- **PRINCIPLES** "Compaction, never compression" — the constraint is on what the compactor may *call*, not what its request may *name*.
- **README** \`lernie dispatch compactor\` and **USER_STORIES US-18** acceptance — same distinction in user-facing terms.

Terminology: the pairing is **declared / callable**, plain English, defined at its point of introduction (ARCH §2.7) per CLAUDE.md. "Capability" was deliberately not used as the noun — TAXONOMY already coins *capability grant* / *capability manifest* for the v1.1 wasm sandbox (§3.6) and records three conflicting field senses. No banned term (bare "call", "turn", "session", "compression"-as-context-management) appears in the added text.