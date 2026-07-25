+++
title = "overflow: summarize is inert at assembly — wire it to trigger the §6 compaction checkpoint"
created = 1784698286
updated = 1784955440
claimant = "Aphid"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Surfaced by bl-e0cb (§5.2 context assembly). The manifest's `overflow: summarize` policy sheds nothing at assembly: assembly is a pure function of the read-state tree (§5.1) and cannot invoke a model, so `src/prompt/dispatch/assembler/body.rs::fit` rides the body whole.

DECIDED (Aphid, 2026-07-25): **subtract `summarize` from the OverflowPolicy vocabulary** — option (b). Rationale, stated against the code rather than the ball's framing:

The premise that the §6 checkpoint is dead mechanism is **false** — the v0.3 terminal-compaction stub was deleted by bl-9dbd, and the checkpoint is genuinely live: `compactor::checkpoint::{state,due}` derives commit count / elapsed seconds from git and answers a pure predicate; `dispatch/child_result/flush.rs::run_flush` runs it at every `lernie advance` step boundary and forks a real compactor child (real model call via `bz`, `write_summary`/`mark_for_deletion`); `compactor_return` lands the compaction merge. Wiring `summarize` would fire *live* mechanism. Subtraction is still right, for three reasons the seam being live does not answer:

1. **Single source of truth.** Model-driven shedding is already declared, once, in `workflow.yaml` `compaction:` (trigger + n). `overflow: summarize` would be a second, per-role home for the same fact ("compact this branch under pressure"). Two representations of one fact drift.
2. **It would puncture assembly's purity.** ARCH §5.1 / PRINCIPLES "Context assembly is deterministic": assembly is a pure function of the read-state tree. Triggering from `fit` requires assembly to emit an out-of-band signal into the step-boundary interpreter — a new channel out of a pure function, i.e. added mechanism, for a capability the `compaction:` clock already has.
3. **It reads the wrong gauge.** §5.2 scopes `budget_tokens` to pinned + body and puts the transcript *outside* it, precisely so the body stays stable between rebuild points; §5.2 says in as many words that "the transcript's pressure valve is compaction, not assembly". A body over budget is what `drop_oldest_summaries` / `truncate` / `drop` exist for. `summarize` would fire the compactor on the one signal that is not the compactor's material.

What subtraction does not solve, and where it would go: if budget pressure should ever drive the clock, its home is a new `CompactionTrigger` variant beside `every_n_commits` / `every_t_seconds` / `on_flush` in `workflow.yaml` — one place, config not code at the call site — never an overflow policy. ARCH §5.2 records that as the future home so the option is not lost.

Landed: `OverflowPolicy::Summarize` removed from `src/config/manifest.rs`, its no-op arm removed from `assembler/body.rs::fit`, `schemas/manifest.json` regenerated, ARCH §5.2's Overflow-policies bullet rewritten. Related observation filed separately: `drop_oldest_steps` is inert by the identical argument.