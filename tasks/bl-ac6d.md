+++
title = "release 0.0.5 carrying the fork-dialog prune (bl-5a36) to crates.io"
created = 1785732134
updated = 1785732135
claimant = "release-driver"
root_commit = "12899370c9ec7a5ed7f8e26d3d4fb914ea6c3310"

[[blockers]]
id = "bl-7a63"
on = "close"
+++
Ship 0.0.5 via the repo release flow (release-plz, hands-off CI publish via trusted publishing — see .github/workflows/release-plz.yml). The release carries exactly one change since 0.0.4: bl-5a36, the fork-dialog prune — children no longer inherit the fork point's messages/**, summary/**, skills/** in their dispatch commit (the yog bl-d023 runaway); the compactor and --from roots are exempt. bl-5a36 landed on main at feeb127 with its CHANGELOG [Unreleased] bullet already in place. Flow per release-plz.yml header: push main (done, 27bcc7d), land 'make promote-changelog VERSION=0.0.5' on main via the task-worktree flow (prep ball, mirroring bl-468e for 0.0.4), then merge the release PR; the CI-gated release job tags v0.0.5 and publishes to crates.io. Close this ball only after the crates.io index actually serves 0.0.5.