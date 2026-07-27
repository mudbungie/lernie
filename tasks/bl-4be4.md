+++
title = "gate: docs"
created = 1785125509
updated = 1785129529
claimant = "Oakum"
parent = "bl-ee80"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-ee80"
on = "claim"
+++
Gate: docs for bl-ee80 — **PASS**; docs updated in the same commit 6d4412b.

- `docs/ARCHITECTURE.md` §2.3 *Silent death is derived* now states both
  death shapes: "whose latest step's model call **never settled
  complete** ... `response.json` closed without a terminal `end`
  (killed or stopped mid-stream, §2.9), or its final attempt segment
  terminated in an `Error` (retries exhausted, or a non-retryable
  error, §2.10 — the segment closes with its clean `end`, so
  absence-of-`end` alone would misread this branch as idle)".
- §8 silent-death sweep bullet now records the naming:
  "Every candidate is **named** in the scan report, not merely counted:
  a root gets no `died` deposit (no parent inbox), so its name in the
  report is the one place an operator learns which branch went quiet."
- §2.10 records the `lernie message` advisory (queue-with-warning,
  never a decline).
- `README.md`: the messaging section gained "**A failed branch is named,
  never refused**" and the `lernie scan` section now describes both
  death shapes and the named report line.
