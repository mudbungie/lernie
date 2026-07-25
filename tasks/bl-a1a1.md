+++
title = "overflow: summarize is inert at assembly — wire it to trigger the §6 compaction checkpoint"
created = 1784698286
updated = 1784959179
claimant = "Aphid"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Surfaced by bl-e0cb (§5.2 context assembly). The manifest's `overflow: summarize` policy sheds nothing at assembly: assembly is a pure function of the read-state tree (§5.1) and cannot invoke a model, so `src/prompt/dispatch/assembler/body.rs::fit` matched it to a no-op arm. The ball asked: wire it to the §6 compaction checkpoint, or subtract it from the OverflowPolicy vocabulary. Either way, fix ARCH §5.2's note.

## DECISION (Aphid, 2026-07-25): subtract `summarize` — option (b)

**The dispatching rationale was wrong and is corrected here.** The premise that the §6 checkpoint is a stub is **false**: bl-9dbd deleted the v0.3 terminal-compaction stub, and the checkpoint is genuinely live — `compactor::checkpoint::{state,due}` derives commit-count and elapsed-seconds from git and answers a pure predicate; `src/prompt/dispatch/child_result/flush.rs::run_flush` evaluates it at every `lernie advance` step boundary and forks a **real compactor child with a real model call**; `compactor_return` lands the compaction merge. Wiring `summarize` would have fired *live* mechanism, not dead scaffolding.

Subtraction is still correct, on three grounds the seam being live does not answer:

1. **Single source of truth.** Model-driven shedding is already declared exactly once, in `workflow.yaml` `compaction:` (§6). `overflow: summarize` would be a second, per-role home for the same fact; two representations of one fact drift.
2. **It would puncture assembly's purity.** ARCH §5.1 / PRINCIPLES "Context assembly is deterministic": assembly is a pure function of the read-state tree. Triggering from `fit` requires that pure function to emit an out-of-band signal into the step-boundary interpreter — new mechanism for a capability the checkpoint clock already has.
3. **Wrong gauge.** §5.2 scopes `budget_tokens` to pinned + body and puts the transcript *outside* it deliberately, stating "The transcript's pressure valve is compaction, not assembly." A body over budget is exactly what `drop_oldest_summaries` / `truncate` / `drop` exist for. `summarize` would fire the compactor on the one signal that is not the compactor's material.

**What subtraction does not solve, and where it would go:** if budget pressure should ever drive the clock, its home is a further `CompactionTrigger` variant beside `every_n_commits` / `every_t_seconds` / `on_flush` in `workflow.yaml` — one place, config rather than a second policy vocabulary. ARCH §5.2 now records this so the option is not lost.

## IMPLEMENTED STATE (complete)

All edits are committed. Files changed:

- `src/config/manifest.rs` — removed the `Summarize` variant from `OverflowPolicy`; enum doc now states that every variant is an act assembly can perform on the tree it holds. Added test `rejects_retired_summarize_overflow` pinning that a manifest naming `summarize` is **declined at parse** (a `LoadError::Yaml` naming the variant), not silently ridden. Removed `"summarize"` from `accepts_each_overflow_variant`.
- `src/prompt/dispatch/assembler/body.rs` — removed the `OverflowPolicy::Summarize => body` no-op arm; `fit`'s doc comment explains that model-driven shedding is not in the vocabulary and belongs to the §6 checkpoint.
- `src/prompt/dispatch/assembler/body/tests.rs` — removed `summarize_sheds_nothing_at_assembly`.
- `schemas/manifest.json` — regenerated with `make schemas` (drops `summarize` from the enum; picks up the new doc text). The golden test passes.
- `docs/ARCHITECTURE.md` §5.2 — the "Overflow policies" bullet rewritten: records the subtraction, the three grounds, that a manifest naming `summarize` is declined at parse, and names the future home (a further `compaction: trigger:` variant).

Checked and clean: no other doc or file names the overflow vocabulary. `README.md`, `docs/PRINCIPLES.md`, `docs/TAXONOMY.md` have no mention; `template/manifest.yaml` uses only surviving policies (`drop_oldest_summaries`, `truncate`). The sole remaining `summarize` string in `src/` is the assertion inside the new rejection test. (`tests/dispatch_tool_e2e.rs:94` has an unrelated goal string "summarize the parent branch's commits".)

## VERIFICATION STATUS

`make check` (fmt-check, clippy -D warnings, tarpaulin --fail-under 100) passed **clean at 100.00% (4350/4350)** in this worktree on the pre-merge tree. Post-merge verification runs as part of the close gate.

## WORKTREE / BRANCH STATE

- Worktree: `/home/u/.local/state/balls/plugins/bl-delivery/home/u/dev/lernie/bl-a1a1`
- Branch: `work/bl-a1a1`, two commits ahead of main:
  - `2ed42d0` "Subtract `summarize` from the OverflowPolicy vocabulary [bl-a1a1]" — **the full decision rationale lives in this commit message**, and is duplicated above.
  - `2a175fe` "Merge branch 'main' into work/bl-a1a1" — clean merge, no conflicts, brings in bl-1c2e (flake cures), bl-4a6c, bl-2503.
- Working tree clean. Nothing uncommitted.

## REMAINING STEPS TO CLOSE

1. From the worktree: `git merge main --no-edit` (re-merge if main advanced again; the last merge was clean).
2. From `/home/u/dev/lernie` (repo root, NOT the worktree):

       PATH=/tmp/claude-1000/-home-mark-dev-lernie/fe8c0ced-96b4-45ac-8547-30d9edb8e1e3/scratchpad/bz003/bin:$PATH \
         bl close bl-a1a1 --as Aphid -m "<summary>"

   The PATH pin matters: `~/.cargo/bin/bz` is 0.0.4 while `Cargo.toml` pins `=0.0.3`, and the gate's five e2e wire tests fail on the version handshake mismatch. `bz003/bin` (and an equivalent `bzroot/bin`) hold a locally built brazen 0.0.3. If those scratch dirs are gone, recreate with `cargo install brazen --version =0.0.3 --root <dir>`.
3. Gate children: **bl-a1a1 has none** — it predates the gate-ball convention. Confirmed via `bl list --json` (zero tasks with `parent: bl-a1a1`). Nothing to sweep after the close.

## DELIVERY HISTORY (why this took several attempts)

Five earlier `bl close` attempts failed at the delivery gate, none attributable to this diff. Root causes, now addressed upstream:

- 2x killed mid-run by SIGTERM (`make: *** [coverage] Terminated`). `src/prompt/stop/cascade.rs` signals whole process groups via `libc::kill(-pgid, SIGTERM)` on a pgid discovered from `/proc`; under load a spawned executor can still show its parent pgid, which under `make check` is the tarpaulin process group, so the stop tests SIGTERM their own run. Filed as **bl-5f0c**.
- 1x `prompt::tool::tests::errors::spawn_retries_past_transient_etxtbsy` (833 passed, 1 failed) — ETXTBSY retry budget vs. a ~40ms file hold, load-sensitive by construction.
- 2x the coverage flake with bl-1c2e's exact recorded signature: 99.98%, 4349/4350, single uncovered line in `src/prompt/tests/advance.rs` (33/34) — a file this diff never touches.

**bl-1c2e has since LANDED on main** and root-caused all of these (advance.rs:116 wall-clock-deadline sleep vs. retry count; bash poll interval; ETXTBSY budget), so they are cured by the merge already in `2a175fe`.

## FOLLOW-UP OBSERVATION (not actioned — out of scope, worth its own ball)

`drop_oldest_steps` is inert by the identical argument used to subtract `summarize`: step records live at `<workspace>/steps/`, outside every worktree (§2.2, §2.3), so the policy names material that cannot be present and `fit` matches it to a no-op arm. It survives only as declared backward compatibility with pre-v0.3.1 manifests, which is arguably vacuous pre-0.0.1. Subtracting it would leave a vocabulary in which every policy operates. Left alone here rather than silently override a prior deliberate decision.