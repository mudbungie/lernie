+++
title = "gate: docs — docs reflect current state"
created = 1784337201
updated = 1784525188
parent = "bl-06d5"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## VERDICT (Costs, 2026-07-19): NOT SATISFIED — one concrete fix owed + one user decision

Reviewed the docs of bl-06d5's undelivered WIP (commit `42f1660` on `work/bl-06d5`,
which is NOT on main — `git log --all --grep=bl-06d5` is empty; main has no
`scripts/` and no `smoke` target).

### What is correct
The README paragraph the WIP adds after "First-run smoke test (required)"
(README.md ~line 157) covers the parent's stated docs requirement in full:
how to run (`make smoke`), what it proves (exit 0 AND no `"type":"error"` event
AND an assistant `content_delta`, read from observable state), why it is NOT in
`make check` (the suite mocks the wire), and that it needs a configured `bz`
credential for `anthropic` (`bz --login --provider anthropic`, or
`ANTHROPIC_API_KEY` / `BRAZEN_API_KEY`) and spends money. The Makefile recipe and
`scripts/smoke.sh` header carry the same explanation.

Re-verified the WIP's factual claims against CURRENT main (the worktree is 3 days
stale — main has since advanced through bl-6d83, bl-f72b, bl-231c):
- `lernie new` and `lernie prompt` both still exist (`src/cmd/new.rs`, `src/cmd/prompt.rs`).
- Every path the `make smoke` recipe feeds the script exists on main:
  `install/models.yaml`, `schemas/tools`, `skills`.
- `claude-sonnet-5` is still the authored worker default (`install/models.yaml`,
  `src/config/tests/providers_split.rs:72`).
So main's drift has NOT invalidated the prose. No stale claim found.

### GAP 1 (concrete, fixable now, owed inside work/bl-06d5)
`make smoke` is missing from README's **### Build targets** table (README.md
lines 749-765). That table lists every other target — `build`, `release`, `test`,
`coverage`, `lint`, `fmt`, `fmt-check`, `schemas`, `new-workspace`, `check`,
`ci`, `install-hooks`, `install`, `uninstall` — and the WIP's only README hunk is
the prose paragraph at ~line 157. A reader scanning the target table will not
learn `make smoke` exists. Fix: add a row, e.g.
`| `make smoke` | Live-wire smoke test: one real `lernie prompt` against the shipped defaults; needs a `bz` anthropic credential and spends money; NOT in `check` |`
This edit is NOT made here: it belongs in `work/bl-06d5` alongside the target it
documents. Adding it on main would document a target main does not have.

### GAP 2 (blocked on the user — this is why the gate cannot close)
bl-06d5's own OPEN QUESTION is unresolved, and its answer rewrites these docs.
The README paragraph, the Makefile comment, and the `smoke.sh` header all
hard-code the shipped defaults: "provider `anthropic`, model `claude-sonnet-5`".
If the user takes option (b) — a provider-agnostic `SMOKE_PROVIDER`/`SMOKE_MODEL`
override — all three prose blocks state the wrong contract and must be rewritten.
The docs therefore cannot be finalized before that decision lands.

### To close this gate
1. User settles bl-06d5's open question (credential, or provider-agnostic override).
2. Whoever finishes bl-06d5 applies GAP 1 (and, if option (b), rewrites the three
   default-naming prose blocks) in `work/bl-06d5`.
3. Re-run this gate against the final diff.