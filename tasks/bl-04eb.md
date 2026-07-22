+++
title = "smoke.sh says 'conversation id' — a retired structural term (ARCH 2.1); it is an agent id"
created = 1784525272
updated = 1784698236
claimant = "Prostheses-smoke"
parent = "bl-06d5"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Why (found by the bl-1de0 alignment gate, 2026-07-19)

`scripts/smoke.sh` (in the bl-06d5 WIP, commit 42f1660) uses the retired structural term **conversation** in two failure strings:

```
[ -n "$ID" ] || fail "lernie prompt printed no conversation id"
case "$ID" in */*) fail "conversation id contains a slash: $ID";; esac
```

`docs/ARCHITECTURE.md` §2.1, **Retired structural terms**, verbatim:

> **"conversation" / "conversation repo"** — the former primitive and its repo. The living instantiation is an **agent**; its container is a **workspace**. Where a later section still says "conversation" / "conversation repo," read *agent* / *workspace*.

`docs/TAXONOMY.md` concurs: *"the ladder retires the structural terms lernie once used: **conversation** and **conversation repo** (superseded by *agent* and *workspace*)"*.

The code already agrees — `src/prompt/mod.rs:256`: *"Returns the agent id"*.

## Deliverable

Both strings → **agent id**, matching the surrounding vocabulary (`agents/<agent-id>`, `steps/<agent-id>/NNN/`).

Docs-only, zero new Rust, no coverage impact. Gates bl-06d5's close.

## Worth a sweep while here

`LERNIE_CONV_REPO` / `LERNIE_CONV_BRANCH` (ARCH §3.3, cited at ARCH:462) carry the same retired `CONV` stem in a *shipped env-var name*. Out of scope for this ball — renaming a public env var is a separate decision — but it is the same finding one layer down, and worth filing if it has not been.