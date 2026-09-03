+++
title = "a prepared start cannot be fanned, and no candidate can be accepted or retired"
created = 1788397354
updated = 1788397354
priority = 3
root_commit = "3efc0d263898c425a0ff2bb042938233e838f436"
+++
Three `control`-classed ops with no interactable here: `fan`, `deliver`, `retire`.

The seat can prepare a start and fire it (`act:prepare act:prompt` on the start control), which is the single-attempt path. The n-candidate path is unreachable: nothing spreads a prepared start over isolated attempts, nothing accepts one of the candidates that come back, and nothing releases a candidate's worktree afterwards. Accepting one of n is an operator judgement, which is why yog classes `deliver` `control` rather than machine (docs/PARITY.md §10).

Exemption rows in `parity.toml` cite this ball until the surfaces land.