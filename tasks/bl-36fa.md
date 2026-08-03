+++
title = "agent-eval reports quality, wall time, attempts, tools, and usage"
created = 1785649843
updated = 1785723896
claimant = "Fathom"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Source: yog bl-e249's Claude Code comparison.

## Why

The repository already owns the 50-task `agent-eval` suite, an injectable external agent driver, pass@1/pass@5, and failure bundles. The Claude Code comparison can support causal hypotheses from source, but cannot claim higher solve rate or lower wall time until two harnesses run the same model, tasks, repositories, and starting commits. Creating another benchmark system would duplicate the existing authority.

## Deliverable

Extend the external eval crates/report so a baseline/candidate run can compare quality and efficiency:

- preserve pass@1/pass@5 and Wilson intervals;
- record outer wall time, model attempts, tool invocations, and all four canonical usage counters when the driver exposes a valid lernie bundle/report;
- report per-task, per-category, and total baseline→candidate deltas;
- print the reproducibility inputs: suite revision, experiment/config, driver command+version, model/provider, run count, and starting fixture identity;
- distinguish missing metrics from zero; never infer price or fabricate token counts;
- no central telemetry and no network/model call in deterministic tests;
- live-model runs remain an explicit operator command with stated run count and spend, never CI;
- use fake drivers/bundles for 100% coverage and update ARCH/README/TAXONOMY as needed.

A later operator run may add a Claude Code driver. This task builds the neutral measurement contract; it does not invoke paid inference.