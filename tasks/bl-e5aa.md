+++
title = "fleet: drop the bl-a900 grant-union workaround, update README/comments for the five landed fixes, re-run the live e2e green"
created = 1785474464
updated = 1785474473
claimant = "Fixer-e5aa"
priority = 2
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
bl-475a/bl-4231/bl-5a1f/bl-a900/bl-e3f5 all landed on main. The fleet config still encodes the pre-fix world: fleet/providers.yaml grants worker slack_read with a comment asserting 'a role's grant must be a subset of its dispatcher's' (no longer true — bl-a900 derives descriptors from the governing config commit), and fleet/README.md (~line 169) explains the union grant. Drop slack_read from worker, rewrite the comment to state the fixed contract (least privilege restored: the coordinator holds slack_post alone; sensors read). Update README GAPS/mapping wherever it hedges on the fixed defects (one-speaker now structural at execution too, per bl-5a1f). Audit fleet/test.sh: every assertion must assert the CORRECT behavior (none may encode a bug-era expectation). Then rebuild and run the live e2e once against main — all scenarios must PASS; any new defect gets reported, not fixed.