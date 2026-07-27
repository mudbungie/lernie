+++
title = "gate: docs"
created = 1785124069
updated = 1785125437
claimant = "Cringle"
parent = "bl-d32a"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-d32a"
on = "claim"
+++
PASS, with one correction made rather than noted.

README gained an 'Auto-push hook' section (what it does, why reference-transaction and not post-commit, what it ignores, why it can never block a landing, the git >= 2.28 floor, what tests/hooks.rs proves), cross-linked from Workflow and Contributor setup.

CORRECTION: the hook comment and README justified the mechanism with a census of main's history ('199 of 199 ... zero merge commits'). main took a merge commit the same day. Replaced with the durable fact: bl delivers by plumbing, so no commit or merge hook fires on a landing.

Correctly untouched: ci.yml, release-plz.toml, the three spec docs (alignment gate bl-ea16), Cargo.toml exclude (already carries /.githooks and /tests).

Gate: make check green, 100% coverage, install contract passed.