+++
title = "make smoke: provider/model override + local ollama support"
created = 1784698625
updated = 1784698625
parent = "bl-06d5"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
## Decision (user, 2026-07-21)
bl-06d5's open question is settled BOTH ways: smoke gains an override (this ball), and the anthropic shipped-default validation is tracked separately (see the standalone anthropic-setup ball).

## Deliverable
- SMOKE_PROVIDER / SMOKE_MODEL inputs on 'make smoke', passed through to scripts/smoke.sh. Unset => today's behavior, byte-for-byte: the pure prime-seeded shipped default (anthropic / claude-sonnet-5). Severability: the override is an input, not a code path fork.
- Mechanism: when set, lay a models.yaml override into the primed harness root via the existing config-root template/ override mechanism (bl-e795) before 'lernie new' — no new lernie flags or verbs; smoke.sh composes existing signals. Verify how the template override actually resolves (src/cmd/new.rs, ARCH) before writing it.
- Document local ollama (bz provider row 'local') as a supported override target, e.g. make smoke SMOKE_PROVIDER=local SMOKE_MODEL=<pulled-model>. Confirm what bz needs for ollama (no credential, server must be serving).
- Rewrite the three default-naming prose blocks (README smoke section, Makefile recipe comment, smoke.sh header) per bl-d9fe GAP 2: defaults stay the shipped anthropic pair, override documented, credential note scoped to the default.

Delivers into work/bl-06d5 (close-gates the parent).