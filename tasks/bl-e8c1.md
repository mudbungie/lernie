+++
title = "gate: tests"
created = 1785129927
updated = 1785130075
claimant = "Quoin"
parent = "bl-8f7c"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-8f7c"
on = "claim"
+++
Gate evidence: actionlint 1.7.12 clean on both workflows after the tag-handoff change; make check green twice (pre-commit hook on commit and on close). Live evidence: run 30231615367 shows the exact failure mode fixed here; the RELEASED_TAG output path is exercised on the next real release (0.0.2), the BACKFILL_TAG path proven by dispatch run 30231841897.