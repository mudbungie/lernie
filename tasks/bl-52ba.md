+++
title = "gate: alignment"
created = 1785124820
updated = 1785129544
claimant = "Parrel"
parent = "bl-f021"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-f021"
on = "claim"
+++
PASS. Checked the landed implementation (main 440da96) against docs/ARCHITECTURE.md, docs/PRINCIPLES.md, docs/TAXONOMY.md.

**Single source of truth (PRINCIPLES).** Verified on main: exactly one caller of \`compactor::builtin_tool_schemas()\` (\`dispatch/tools.rs:86\`) and exactly one caller of \`compactor::refusal\` (\`dispatch/tool_step.rs:79\`); both step drivers — \`dispatch::run_exchange\` (mod.rs:181) and the \`advance\` hop (hop.rs:91) — reach the *same* \`tools::compose\`. That is a net removal of a divergence, not an addition: before this ball the compactor injection lived in \`hop.rs\` alone, so the two drivers disagreed about what a compactor's request declared. "What does this request declare?" now has one home.

**A special case is usually a missing reframe (PRINCIPLES).** The ball was filed as a compactor bug and the architect's leaning was a compactor rule; it shipped as the *general* composer rule instead. Justified, not merely tidier: an ordinary worker hits the same wedge whenever a model invents a tool name, because §2.5 pairing lands the \`tool_use\`/\`tool_result\` in the transcript anyway and every subsequent step on that branch then ships an undeclared name. The general path with the compactor as its standing case; no role branch in the closure. §3.3 states it that way.

**Structural over disciplinary (PRINCIPLES).** §2.7's deletion-only guarantee is preserved by construction, not by convention: \`refusal\` is consulted *before* \`deps.tool_executor.execute\`, so the executor is never entered for a foreign tool — asserted directly by the hop test, which checks the stub executor recorded **zero** invocations. Widening the declaration therefore does not widen the guarantee.

**Severability (PRINCIPLES) / §4.3.** The compactor pair stays procedure-owned — still never a \`providers.yaml\` \`tools:\` list, still never on \`descriptions/**\`. \`Resolved\` gained \`role\`, borrowed from \`WorkerConfig::role\` (its one home, §4.3), so nothing new is stored. No new flag, config key, or verb.

**Transcript immutability (§2.3) / transcript-backed framing (§3.3).** The declaration is widened to fit the history; the history is never rewritten to fit the toolset. The rejected alternative is recorded in §2.7 with its reason.

**TAXONOMY.** No new coinage. The pairing is **declared / callable** — plain English, defined at introduction (ARCH §2.7). "Capability" was deliberately avoided as the noun: TAXONOMY §"Action, capability, plugin" already coins *capability grant* / *capability manifest* for the v1.1 wasm sandbox and records three conflicting field senses (marketing umbrella, MCP protocol feature flag, Anthropic's Skills framing). No banned term appears in the added prose.