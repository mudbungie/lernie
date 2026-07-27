+++
title = "release-binaries resolves the tag via git describe from the pinned push sha — races main and misses the just-cut tag"
created = 1785129927
updated = 1785129927
priority = 9
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"
+++
Found watching bl-a124's live close push (run 30231615367): the whole publish chain went green — crates.io lernie 0.0.1 published via trusted publishing, tag v0.0.1 pushed, GitHub Release cut — but release-binaries failed: 'fatal: No tags can describe 524986a'. bl-c786 landed on main between the push and the release job's branch checkout, so release-plz tagged main's NEW tip (38efef3), a DESCENDANT of the run's pinned sha (524986a) that the binaries job checks out; git describe only sees ancestor tags.

Fix (single source of truth): the release job already knows the tag — release-plz/action's releases output is [{"package_name":"lernie","prs":[],"tag":"v0.0.1","version":"0.0.1"}]. Extract .[0].tag into a job output and have release-binaries consume it via needs, instead of re-deriving with git describe. binaries_tag dispatch input keeps precedence for backfill.

Done when: the resolve step reads the tag from needs.release-plz-release.outputs, actionlint clean, gate green; then a workflow_dispatch with binaries_tag=v0.0.1 backfills the missing lernie-x86_64-unknown-linux-gnu.tar.gz onto the existing v0.0.1 Release.