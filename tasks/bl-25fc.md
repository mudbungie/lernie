+++
title = "runner: agent-eval --config/--suite/--runs (v0.10)"
created = 1783917759
updated = 1784009423
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-6a3b"
on = "claim"
+++
## Scope
The agent-eval runner (§9.3 / v0.10): `agent-eval --config <experiment> --suite <suite> --runs N` executes an evaluation run (experiment × suite × N) and reports per-task metrics. Home is a separate binary in this workspace, `crates/agent-eval` (keeps experiment-driver concerns out of the lernie core).

## Deferred by bl-8094
Filed by the evaluation-layer epic (bl-8094), which shipped bundle/replay (§9.2) and the task suite (`tests/suite/`, 50 tasks, ≥10 per §9.1 category, validated by `tests/suite.rs`). The runner was deferred because it only becomes meaningful once workflow variants exist — hence `--needs bl-6a3b` (workflow-action interpreter). Building a runner against nonexistent workflow variants was explicitly out of scope.

## Runner contract (§9.3, carried forward)
- Input: an experiment (a `workflow.yaml` variant under `experiments/<name>/`, §9.3 — a config diff, no code changes) × the suite (`tests/suite/`) × N runs (N≥5).
- Per task: seed a fresh workspace under an isolated `LERNIE_HOME`, run the task `setup` (shell), dispatch the agent with the task `prompt`, then run the task `check` (shell) — exit 0 is the sole pass signal (success decided by observable state, never the agent's claim). The suite format is documented in `tests/suite/README.md`.
- Output: per-task pass@1 and pass@5, plus per-category failure breakdowns (the seven §9.1 tags).

## Statistics (§9.1, must implement)
- pass@1 (primary): mean per-task pass rate. Per task compute #passes/N; mean across tasks (mean-of-means, N fixed per task). Report 95% Wilson score intervals on the mean. Captures reliability.
- pass@5 (secondary, any-of-5): per task, did any of 5 runs pass; report fraction of tasks. Captures ceiling capability.
- Optimization target is pass@1; pass@5 distinguishes capability shifts from reliability shifts.
- v0.9 baseline check: baseline harness ~40% ± 5% pass@1 on the suite (Wilson CI).

## Experiments (§9.3)
`experiments/<name>/workflow.yaml` variants (baseline, strict-verifier, parallel-workers, …). A new experiment is a config diff; deployable without code changes (v0.10: under 60s end-to-end).

## Notes for the implementer
- Replay/bundle (`lernie bundle` / `lernie replay`, §9.2, shipped by bl-8094) archive and reconstruct a run subtree for post-hoc inspection; the runner can bundle failing runs for triage.
- crates/agent-eval was intentionally NOT scaffolded by bl-8094 (an empty crate cannot meet the 100% coverage gate); create it here.