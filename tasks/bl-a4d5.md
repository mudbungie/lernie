+++
title = "the prompt surfaces carry pre-ladder vocabulary the taxonomy retired: 'subagent' (deleted as a category), 'conversation' (superseded by agent/workspace), and a bare 'per-call' in schemas, skills, and souls"
created = 1785906111
updated = 1785906115
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
`docs/TAXONOMY.md` L290 states lernie **deletes "subagent" as a category** — "a child agent is just an agent" — and retires **conversation** (superseded by *agent* and *workspace*). The prompt surfaces never got the memo. These strings are shipped context: every model the harness drives reads them, so they teach the retired vocabulary back to the agents on every dispatch.

Sites found 2026-08-04 (verify against the tree; more may exist):

- `schemas/tools/dispatch.json:13` — "Per-call goal text ... the **subagent's** goal.md ... The **subagent** reads its goal there ... the **subagent's** intermediate work". Also carries a bare "per-call" in the model-interaction sense, which is the sense ARCHITECTURE 2.1 still bans (the bl-1966 carve-out sanctions only the programming sense: call site, callback, callee, function call, system call).
- `schemas/tools/dispatch.json:17` — "identities and tasks in a **subagent** tree". Landed with bl-404d; the ball body itself specified this wording, so this site needs the operator's ruling, not a silent fix.
- `skills/dispatch/SKILL.md:30` — "keeps **subagent** identities and".
- `schemas/workflow.json:82` — "**Per-conversation** spend limits", and "any driver (root or **subagent**)".
- `template/souls/worker.md:4` — "a tool call targeting a **subagent**".

Deliverable: one ruling, applied consistently. Either the taxonomy's deletion holds and every site is revoiced (*child agent*, *agent tree*, *per-agent*, *per-dispatch*), or the taxonomy carves out an exception and says why. Do not split the difference silently — the point of the ladder is that one term has one home.

The pinning tests move with the strings: `src/install/tests/toolspec.rs` asserts the dispatch schema's descriptions.

Discovered by the bl-404d rescue, which flagged it rather than acting on it because the ball body specified the wording.