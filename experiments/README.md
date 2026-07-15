# Experiments (ARCH §9.3)

An **experiment** is a `workflow.yaml` variant — a config diff, no code
changes. Each lives in its own directory:

```
experiments/
└── <name>/
    └── workflow.yaml
```

The evaluation runner (`agent-eval --config <name> --suite <suite> --runs
N`, ARCH §9.3 / v0.10) resolves `--config <name>` to
`experiments/<name>/workflow.yaml` and runs the suite (`tests/suite/`,
§9.1) against it N times per task, reporting per-task and per-category
pass@1 (with 95% Wilson intervals) and pass@5.

## Adding an experiment

Copy an existing `workflow.yaml`, edit the bindings (ARCH §6), and drop it
under a new `experiments/<name>/`. No code changes are needed — a new
experiment is deployable in under 60 seconds end-to-end (v0.10). The
`workflow.yaml` schema is the same one `lernie` reads from a config commit
(ARCH §2.2); see `template/workflow.yaml` for the annotated reference.

## Shipped experiments

| Name | What it is |
|---|---|
| `baseline` | The default workflow (`template/workflow.yaml`): the v0.9 baseline harness against which variants are measured (§9.1, ~40% ± 5% pass@1 target). |

Variants that beat baseline on a failure category by a statistically
significant pass@1 margin are the v0.11 milestone (ARCH §12); they slot in
here as new directories.
