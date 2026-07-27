+++
title = "commit-identity guard test + changelog normalization"
created = 1785125363
updated = 1785129712
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Follow-on to the 2026-07-26 main history rewrite (identity normalization: every author+committer on main is now mudbungie <mudbungie@gmail.com>; all Co-Authored-By trailers stripped; old tip 2c942f9 -> new tip cd8aa45, tree-identical).

Two deliverables:

1. tests/commit_hygiene.rs — a this-machine-only guard that keeps main clean going forward. Gated on a marker file OUTSIDE the repo ($XDG_CONFIG_HOME/lernie/enforce-commit-identity, default ~/.config/lernie/enforce-commit-identity): absent => the test passes trivially, so public CI and other machines are unaffected. Armed => walks git log refs/heads/main and asserts for every commit that author and committer are mudbungie <mudbungie@gmail.com> or github-actions[bot] (the bot must stay allowed: release-plz authors release commits as it), that the message carries no Co-Authored-By (case-insensitive), and that none of author/committer/message mentions t@t.local or orionriver. Runs git via std::process::Command; no new dependencies; skips (passing) if git or the repo is unavailable. Policy lives in the marker file: delete it and the guard is off, no code edit.

2. CHANGELOG.md normalization — one uniform bullet per shipped change under the 0.0.1 section: duplicates collapsed, bl-gate process commits (bare "tests"/"docs"/"alignment") dropped, giant multi-clause subjects reduced to a single summary clause, [bl-xxxx] refs preserved (they are the task trail), no stale short shas invalidated by the rewrite. release-plz.toml [changelog] gains a skip parser for gate commits so FUTURE entries come out uniform without hand-editing.