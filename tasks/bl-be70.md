+++
title = "USER_STORIES.md: record the 2026-07-27 live walk — upgrade live-verified items from unit-only/unchecked, note the two non-findings"
created = 1785133371
updated = 1785133371
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Why

docs/USER_STORIES.md is the promise suite 0.0.x is evaluated against, and it
records which acceptance items are integration-proven vs unit-only vs never
checked. The 2026-07-27 evaluation walk (fresh build of main, isolated
LERNIE_HOME, live wire via local ollama qwen3.5:9b) verified a batch of items
the doc still lists as unchecked. Record them so the next pass does not
re-walk proven ground. Verify each claim against the walk record below before
editing — do not upgrade anything not listed.

## Live-verified by the walk (upgrade to integration-proven, cite this ball)

- US-11/US-12: `lernie tool load_skill` and `lernie tool message` as real
  subprocesses — loaded/already_loaded with the body materialized; deposited
  with create-only inbox file; unknown-skill decline names the pool. (§12
  listed both as "not checked by any pass".)
- US-13: message on a quiescent root — ~4ms, exit 0, detached advance delivers
  and steps within seconds (verified twice).
- US-16: live SIGTERM mid-model-call — executor exits 0, response.json ends
  without terminal `end`, id still printed. `--stop-children` felled a
  genuinely running child (bz process and driver gone, `epitaph: stopped` +
  `terminal_ref:` deposited into parent inbox); bare stop on a quiescent
  parent left a separate live child running (correct scoping).
- US-17: live dispatch→child full step loop→deposit (`epitaph:
  final-response` + `terminal_ref:`)→automatic revival of the quiescent
  parent, no `lernie scan` needed (delivery commit + further model-output
  commit with no explicit trigger).
- US-05: `--from` decline-leaves-no-branch-behind; EDITOR-unset vi fallback
  fails cleanly non-interactively.
- US-04: nested non-existent parent dirs created (undocumented but works).
- US-19/US-20: bundle of a parent includes hyphen-descendant child ref +
  governing lineage; bundling into a pre-existing non-empty out-dir works;
  replay reconstructs a fully driveable workspace; missing-archive decline is
  clean and specific.
- US-06: full live `lernie prompt` — exact 4-commit history, control files
  absent, messages/002 JSON as specified, content_delta present, no error
  event.

## Non-findings worth recording (so they are not rediscovered)

- `lernie bundle` including a SIBLING config branch is intentional: the bundle
  carries every merge-base candidate so replay re-derives governing_config by
  the identical computation (src/workspace.rs `config_lineage` + its test).
  Add a sentence where the doc/README states "every config/* ref whose
  history reaches it" if that phrasing misleads (it did mislead one walker).
- `lernie tool` shims with a malformed LERNIE_CONV_BRANCH (`agents/<id>`
  instead of the bare id per ARCH §3.3) produce a raw-path ENOENT; input is
  harness-controlled, not model-supplied, so this is noted, not a defect.

## Scope

Docs only (USER_STORIES.md, possibly one README sentence). No code. Keep the
doc's existing voice and checked/unchecked notation.