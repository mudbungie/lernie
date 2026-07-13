# Task suite (ARCH §9.1, v0.9)

The evaluation task suite: manually constructed tasks with **machine-checkable
success criteria**, tagged by the failure category each is designed to provoke.
This directory is the single source of truth for what an experiment is measured
against (§9.3) — experiments and the suite version live in the repo together.

The runner that executes the suite (`agent-eval --config <experiment> --suite
<suite> --runs N`, §9.3 / v0.10) is **deferred**: it only becomes meaningful
once workflow variants exist (bl-6a3b). This directory is the runner's input,
authored ahead of it; the well-formedness test (`tests/suite.rs`) is the only
thing that reads it today.

## Layout

One YAML file per failure category, each a list under `tasks:`. A task's file
records its **primary** category; a task may carry **secondary** category tags
when it provokes more than one failure mode (this is how 50 tasks reach the
§9.1 target of ≥10 tasks per category across seven categories).

## Task schema

```yaml
tasks:
  - id: early-termination-01        # unique across the whole suite
    categories: [early_termination] # ≥1 of the seven tags below; first = primary
    prompt: |                       # the goal handed to the agent under test
      ...
    setup: |                        # optional: shell seeding the workspace
      ...                           #   (run before the agent; cwd = workspace)
    check: |                        # shell run in the workspace AFTER the run;
      ...                           #   exit 0 = pass, non-zero = fail
```

`setup` and `check` are ordinary shell. The runner (v0.10) seeds a fresh
workspace, runs `setup`, dispatches the agent with `prompt`, then runs `check`
in the workspace — **exit 0 is the sole pass signal**, so success is decided by
observable state, never by the agent's own claim. A task with no `setup` starts
from an empty workspace.

## Failure categories (§9.1)

The seven tags, each mapping to a §9.1 failure mode:

| Tag | §9.1 failure mode |
|---|---|
| `early_termination` | Early termination — stopping before the goal is met. |
| `scope_reduction` | Scope reduction — delivering a subset of the asked scope. |
| `skipped_tests` | Skipped tests — claiming done without running/passing tests. |
| `hallucinated_apis` | Hallucinated APIs or facts — inventing symbols that do not exist. |
| `error_recovery` | Error recovery failure — not recovering from a seeded broken state. |
| `fabricated_success` | Fabricated success claims — asserting success the artifact contradicts. |
| `context_hygiene` | Context hygiene — needle-in-haystack, compaction, prompt-injection resistance. |

## Metrics (§9.1)

The runner reports, per §9.1:

- **pass@1** (primary) — mean per-task pass rate over N runs (mean-of-means, N
  fixed per task), with 95% Wilson score intervals. Reliability.
- **pass@5** (secondary) — fraction of tasks passing at least one of five runs.
  Ceiling capability.

Optimization target is pass@1; pass@5 distinguishes capability shifts from
reliability shifts. Target baseline pass@1 on this suite is ~40% (§9.1, v0.9).
