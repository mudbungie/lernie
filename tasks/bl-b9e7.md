+++
title = "gate: alignment"
created = 1785129594
updated = 1785129857
claimant = "Quoin"
parent = "bl-a124"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-a124"
on = "claim"
+++
Gate evidence: checked against docs/ARCHITECTURE.md, docs/PRINCIPLES.md, docs/TAXONOMY.md — none mentions CI or release wiring (grep for workflow_run/release-plz/crates.io/ci.yml is empty), so no spec amendment needed. The change follows the repo's principles: single source of truth (CI runs once per push, as the called instance — dedup by subtracting ci.yml's push arm, not by adding concurrency machinery; the pin/toolchain comments in ci.yml untouched) and smallest truthful wiring (no new flags, no new secrets, no crates-io-auth-action). No new terms of art coined.