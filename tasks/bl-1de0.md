+++
title = "gate: alignment — coherent vs ARCHITECTURE/PRINCIPLES/TAXONOMY"
created = 1784337202
updated = 1784525305
claimant = "Aficionado-1de0"
parent = "bl-06d5"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## ALIGNMENT VERDICT (Aficionado-1de0, 2026-07-19): **FAIL** — 3 findings, all in the bl-06d5 WIP (worktree `work/bl-06d5`, commit 42f1660)

Reviewed: `Makefile` (+`smoke` target), `README.md` (+"make smoke" section), `scripts/smoke.sh` (89 lines) against `docs/ARCHITECTURE.md`, `docs/PRINCIPLES.md`, `docs/TAXONOMY.md`.

### What IS aligned (recorded so it is not re-litigated)
- **The deliverable is exactly what ARCH §4.2 mandates.** Verbatim: *"A live `lernie prompt` is therefore the **required smoke test** after `lernie new`: it is the first — and, by this stance, only — check that the authored ids actually resolve on the wire. … this is why the smoke test is a workflow requirement, not an optional nicety."* `make smoke` implements that requirement.
- **Verdict-from-observable-state is PRINCIPLES "Inspectability first" done right**, and grepping `steps/*/response.json` is NOT a violation of PRINCIPLES "Context has one home" (*"step records under `steps/` are diagnostic, with zero runtime content reads"*) — smoke.sh is diagnostic tooling, not a runtime path. ARCH §2.2 is the correct cite for "step record exists off-worktree" (the off-worktree statement lives at §2.2, not §2.3).
- **Severability holds**: `smoke` is out of `check` and out of the close gate — PRINCIPLES "Severability test".
- **No credential is ever seen by lernie** — the script defers entirely to `bz`, per PRINCIPLES "Integrations are external binaries" (*"lernie never sees a credential"*).

### F1 — `scripts/smoke.sh` hand-duplicates `lernie prime`'s seeding (**single source of truth**; blocking)
smoke.sh founds its throwaway harness root by hand:
```
mkdir -p "$HOME_DIR/tools" "$HOME_DIR/skills"
cp "$MODELS_YAML" "$HOME_DIR/models.yaml"
cp "$TOOLS_DIR"/*.json "$HOME_DIR/tools/"
cp -R "$SKILLS_DIR"/. "$HOME_DIR/skills/"
```
ARCH §2.2 makes this one verb's job, verbatim: *"One verb **founds the installation substrate** … `lernie prime` … lays down what a ready installation carries: the default global `models.yaml` (§4.2), the `tools/` schema pool and the `skills/` pool (§3.3), and the empty `workflows/` / `workspaces/` directories. … The shipped assets are **embedded in the binary** … **`make install` invokes `lernie prime`** rather than duplicating the seeding, so the verb is the single source of truth for what a ready installation looks like."* The `make install` recipe already says so in its own comment: *"the seeding lives in one place (the verb), not duplicated here."*

Two costs, not one:
1. PRINCIPLES "Single source of truth" / "Everyone uses the front door" — a second, divergent seeding path that will drift the moment `prime` seeds anything new.
2. **It weakens the test's own purpose.** `prime` seeds from assets **embedded in the binary**; smoke.sh seeds from the **source tree** (`install/models.yaml`, `schemas/tools`, `skills/`). So `make smoke` does not exercise the shipped install path — a wrong id embedded in the binary would be masked by a correct one in the working tree. That is a sibling of the exact bug class (`claude-sonnet-4-7`) this ball exists to catch.

**Fix:** replace the four seeding lines and the three `MODELS_YAML`/`TOOLS_DIR`/`SKILLS_DIR` inputs (Makefile recipe + `: "${...:?}"` guards + header comment) with:
```
export LERNIE_HOME="$HOME_DIR"
"$LERNIE_BIN" prime || fail "lernie prime exited non-zero"
```
Net: smaller script, one input (`LERNIE_BIN`), and the smoke test now covers `prime` → `new` → `prompt`, the real install path.

*Context:* `work/bl-06d5` is **3 commits behind main**; `lernie prime` (bl-6d83, `src/cmd/prime.rs`, main commit 929e261) most likely landed after this WIP was written. Merge main into the worktree first.

### F2 — "conversation id" is a **retired structural term** (terminology gate; blocking)
`scripts/smoke.sh`, two occurrences:
```
[ -n "$ID" ] || fail "lernie prompt printed no conversation id"
case "$ID" in */*) fail "conversation id contains a slash: $ID";; esac
```
ARCH §2.1 **Retired structural terms**, verbatim: *"**\"conversation\" / \"conversation repo\"** — the former primitive and its repo. The living instantiation is an **agent**; its container is a **workspace**."* TAXONOMY §281 concurs: *"the ladder retires the structural terms lernie once used: **conversation** and **conversation repo** (superseded by *agent* and *workspace*)"*. The code already agrees — `src/prompt/mod.rs:256`: *"Returns the agent id"*.

**Fix:** both strings → **agent id**. (Matches `agents/<agent-id>`, `steps/<agent-id>/NNN/`.)

### F3 — bare "call" in README (banned usage; blocking)
`README.md`, new section: *"The **live call** needs a configured `bz` credential for the `anthropic` provider…"*

ARCH §2.1 **Banned usage**, verbatim: *"**\"call\"** without a qualifier — use model call, tool call, or API call."* Also `CLAUDE.md` and PRINCIPLES "Terminology is load-bearing".

**Fix:** → "The live **model call** needs a configured `bz` credential…". The other two diff hits ("the FIRST real model call", "makes the first real model call") are correctly qualified — leave them.

### Disposition (final, 2026-07-19)

Owner check first: bl-06d5 is claimed by **Sorehead-06d5**, but it is **stalled, not in flight** — claimed 2 days ago, last commit 2026-07-17 01:52, clean tree, nothing since; it stopped on its own OPEN QUESTION to the user about the anthropic credential. So the findings were filed rather than deferred to a live owner.

- **F1 → filed as `bl-a785`** (`--parent bl-06d5 --blocks close`).
- **F2 → filed as `bl-04eb`** (`--parent bl-06d5 --blocks close`).
- **F3 → FIXED** at `b64e2b0` on `work/bl-06d5`: "live call" → "live model call", reflowed. Pre-commit green (fmt, clippy, 100% coverage 3978/3978, +0.00%). Applied directly in that worktree because the text exists nowhere else — it is new in the WIP, not on main. Recorded loudly in bl-06d5's journal so Sorehead sees the outside commit on resume.

This gate closes: its findings are now enforced structurally by two close-blockers rather than by an open gate. bl-06d5 remains independently blocked on an anthropic credential, so none of this is the critical path.
