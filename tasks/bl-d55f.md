+++
title = "the agent's name reaches the model through the assembled context, not as prose prepended to the first user message"
created = 1785649286
updated = 1785650554
claimant = "context-home"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-c8ed"
on = "claim"

[[blockers]]
id = "bl-b415"
on = "claim"
+++
## Operator ruling (2026-08-01, filed from yog operation)

Today the dispatcher (yog) tells an agent who it is by prepending `You are <name>.` to the first user message. The operator has ruled this wrong: the first user message is the user's; identity belongs in the context the harness assembles — the same channel that carries the role's system prompt / context files.

## What this ball is

With bl-c8ed landed, the name is a first-class fact stored under the agent. This ball makes lernie surface that fact **model-facing**: when an agent has a name, the assembled context states it. No name fact -> context says nothing (today's behavior, unchanged). Single source of truth: the stored name fact is the one home; the context line derives from it at assembly, never stored a second time.

## Premises verified against the tree (2026-08-02, context-home)

- **bl-c8ed confirmed.** Commit cacca08. The fact is `<worktree>/name`, always written by the dispatch commit (`workspace::agent_name::settle`, called from `prompt::dispatch::trim_to_context`), empty = unnamed. Read from outside via `git show agents/<id>:name` (`agent_name::read`).
- **CORRECTION — the ball cited the wrong ARCH section.** ARCH §2.8 is **Goals**, not "the system slot / committed context". §2.8 does own the name's pinning law (it carried the sentence this ball overturns: "it is display, not instruction: it enters no model call by virtue of being the name"), so it is still the right place to amend — but the **system slot** itself is specified in **§2.3 *Goal and soul are pinned files, not sequence item zero*** and listed in **§5.2 *Structural wire homes***.
- **CORRECTION — `step_commit.rs` is not where the soul "rides".** It composes the slot (`prepend_goal`, now `compose_system`); the two call sites that put it on the wire are `src/prompt/dispatch/mod.rs` (root start) and `src/prompt/dispatch/advance/hop.rs` (every later step).
- **Under the bl-b415 ruling the manifest is the inclusion list** — but no `pinned` entry was needed: the system slot is a *structural wire home*, like the tools array, so it composes regardless of any role's lists. `name` was added to the assembler's structural-home skip set so a role that does pin it cannot send it twice.

## Delivered

`compose_system(goal, name, soul)` -> `<goal>\n{goal}\n</goal>\n\n[Your name is {name}.\n\n]{soul}`. Empty name composes byte-identically to the pre-name slot.