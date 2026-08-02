+++
title = "the agent's name reaches the model through the assembled context, not as prose prepended to the first user message"
created = 1785649286
updated = 1785649286
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-c8ed"
on = "claim"
+++
## Operator ruling (2026-08-01, filed from yog operation)

Today the dispatcher (yog) tells an agent who it is by prepending `You are <name>.` to the first user message. The operator has ruled this wrong: the first user message is the user's; identity belongs in the context the harness assembles — the same channel that carries the role's system prompt / context files.

## What this ball is

With bl-c8ed landed, the name is a first-class fact stored under the agent. This ball makes lernie surface that fact **model-facing**: when an agent has a name, the assembled context (ARCH §2.8 — the system slot / committed context, where the role's soul already rides via `src/prompt/dispatch/step_commit.rs`) states it. No name fact → context says nothing (today's behavior, unchanged). Single source of truth: the stored name fact is the one home; the context line derives from it at assembly, never stored a second time.

## Verify before editing (do not trust this ball blindly)

- Confirm where bl-c8ed actually put the name fact (its body says stored under the agent; expected `agents/<id>/name` or similar) — read the landed code, not this ball.
- Read ARCH §2.8 (assembled context / system slot) and the step_commit path before choosing the injection point. Follow how the soul already reaches the model; do not invent a parallel channel.
- Wording should be minimal identity prose (e.g. `Your name is <name>.` or the ARCH-preferred phrasing) — no instructions attached; lernie's own docs may already pin a phrasing, check first.

## Downstream (out of scope here)

yog will stop prepending the prose stamp once this releases and its pin bumps (yog ball filed, gated on yog bl-08f2). Do not touch yog.

## Discipline

Read lernie AGENTS.md + ARCH. Full coverage, all gates. Amend ARCH where it describes what the assembled context contains.