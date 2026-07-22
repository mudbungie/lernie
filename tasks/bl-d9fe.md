+++
title = "gate: docs — docs reflect current state"
created = 1784337201
updated = 1784698816
claimant = "Prostheses-bad2"
parent = "bl-06d5"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-06d5"
on = "claim"
+++
## VERDICT (Prostheses-bad2, 2026-07-21): SATISFIED — both gaps closed on main (delivery f7b0b2e)

bl-06d5 delivered to main as f7b0b2e "live-wire smoke test ... [bl-06d5]".
Re-verified both recorded gaps against delivered main:

- GAP 1 (make smoke missing from README Build targets table): FIXED. README.md
  line 813 now carries a `make smoke` row in the ### Build targets table.
- GAP 2 (default-naming prose blocks pending the user decision): FIXED. The user
  settled bl-06d5's open question in favor of a `SMOKE_PROVIDER`/`SMOKE_MODEL`
  override (delivered as bl-bad2). All three prose blocks now document it with
  the default kept as the shipped anthropic pair and the credential note scoped
  to that default:
  - README smoke section (README.md ~L194-215): override documented, both-or-
    neither stated, `SMOKE_PROVIDER=local` for ollama shown, credential note
    scoped to the anthropic default.
  - Makefile smoke recipe comment (Makefile L102-107): override + examples.
  - scripts/smoke.sh header (L14-29): default target + override front-door
    explanation (config-root template/ override + pre-prime models.yaml).

No stale claims; the shipped default and the override are both documented
truthfully. Gate satisfied — closing.