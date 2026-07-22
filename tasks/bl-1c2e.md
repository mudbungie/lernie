+++
title = "Coverage measurement flake: intermittent 1-line miss on unrelated diffs (llvm region attribution)"
created = 1784699263
updated = 1784699263
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Observed (2026-07-21, twice, independently)
'make coverage' (tarpaulin, llvm engine, --fail-under 100) intermittently reports exactly one uncovered line that is logically covered, on diffs that never touched it; an immediate retry with zero code change greens. Two same-day sightings:
- bl-19cb agent: after splitting src/prompt/dispatch/child_result.rs, coverage dropped to 4227/4228 with src/prompt/budget/mod.rs:127 ('u64::from(limit)') flagged — a line its own depth test provably executes. Pristine main measured 100%.
- bl-4f6d agent: the same line, after an unrelated payload-drop diff in src/prompt/dispatch/advance*.
- Also that day (bl-06d5 close prep): src/prompt/tests/advance.rs at 33/34 once, green on retry.
Both agents converged on the same mitigation: the flagged line duplicated an expression computed 3 lines up ('u64::from(limit)' at comparison and struct field); hoisting to one binding removed the fragile isolated region. That fix is on main — the SYMPTOM is gone there, the CAUSE is not understood.

## Investigate
1. Root cause: llvm coverage-region attribution shifting under unrelated codegen changes (CSE folding a duplicated trivial expression into a shared region)? A tarpaulin llvm-engine bug? Or environment (the known GIT_DIR / SPAWN_LOCK hook-env traps)? Reproduce if possible: check out the bl-19cb pre-fix worktree state and measure repeatedly.
2. Decide the durable posture: switch measurement (e.g. cargo llvm-cov, as sibling repo brazen uses, reportedly stable at 100% there), pin tarpaulin version/flags, or codify the mitigation as a lint-style rule (no duplicated trivial conversions — which is DRY anyway).

## Why it matters
The 100% floor is a close gate; a flaking gate burns full tarpaulin cycles (~minutes each) on every false red and trains agents to retry-until-green — which would also mask a REAL 1-line regression.

## Deliverable
Root cause written up in this ball; the chosen posture implemented (or explicitly documented as accepted risk with the retry rule stated in README/AGENTS).